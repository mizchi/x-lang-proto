# 非同期Webサーバーの例（エフェクトシステムを活用）

module AsyncWebServer {
  export { startServer, handleRequest, router }
  
  # ============================================
  # 型定義
  # ============================================
  
  type HttpMethod = {
    GET
    POST
    PUT
    DELETE
    PATCH
  }
  
  type Request = {
    method : HttpMethod
    path : Text
    headers : Map Text Text
    body : Maybe Bytes
    params : Map Text Text
    query : Map Text Text
  }
  
  type Response = {
    status : Int
    headers : Map Text Text
    body : Bytes
  }
  
  type Handler = Request ->{Async, IO, Error} Response
  
  type Route = {
    method : HttpMethod
    pattern : Text
    handler : Handler
  }
  
  type Middleware = Handler -> Handler
  
  # ============================================
  # エフェクト定義
  # ============================================
  
  effect Database {
    query : Text -> List (Map Text Value)
    execute : Text -> ()
    transaction : (() ->{Database} a) -> a
  }
  
  effect Cache {
    get : Text -> Maybe Bytes
    set : Text -> Bytes -> Duration -> ()
    invalidate : Text -> ()
  }
  
  effect Logger {
    info : Text -> ()
    warn : Text -> ()
    error : Text -> ()
  }
  
  # ============================================
  # レスポンスビルダー
  # ============================================
  
  ok : Bytes -> Response
  ok body = {
    Response {
      status = 200
      headers = singleton("Content-Type", "text/plain")
      body = body
    }
  }
  
  json : a -> Response
  json data = {
    Response {
      status = 200
      headers = singleton("Content-Type", "application/json")
      body = toBytes(toJson(data))
    }
  }
  
  notFound : () -> Response
  notFound = {
    Response {
      status = 404
      headers = singleton("Content-Type", "text/plain")
      body = toBytes("Not Found")
    }
  }
  
  serverError : Text -> Response
  serverError msg = {
    Response {
      status = 500
      headers = singleton("Content-Type", "text/plain")
      body = toBytes("Internal Server Error: " ++ msg)
    }
  }
  
  # ============================================
  # ミドルウェア
  # ============================================
  
  # ロギングミドルウェア
  loggingMiddleware : Middleware
  loggingMiddleware handler = { req ->
    with logger do {
      let start = { now() }
      info(format("{} {}", req.method, req.path))
      
      response <- handler(req)
      
      let duration = { now() - start }
      info(format("Response: {} ({}ms)", response.status, duration))
      
      return response
    }
  }
  
  # キャッシュミドルウェア
  cacheMiddleware : Duration -> Middleware
  cacheMiddleware ttl handler = { req ->
    if req.method == GET {
      let key = { cacheKey(req) }
      
      with cache {
        match get(key) {
          Some(cached) -> {
            return Response {
              status = 200
              headers = singleton("X-Cache", "HIT")
              body = cached
            }
          }
          None -> {
            response <- handler(req)
            if response.status == 200 {
              set(key, response.body, ttl)
            }
            return response with {
              headers = insert("X-Cache", "MISS", response.headers)
            }
          }
        }
      }
    } else {
      handler(req)
    }
  }
  
  # 認証ミドルウェア
  authMiddleware : Middleware
  authMiddleware handler = { req ->
    match getHeader("Authorization", req) {
      Some(token) -> {
        with auth {
          user <- verifyToken(token)
          let reqWithUser = { req with { user = Some(user) } }
          handler(reqWithUser)
        }
      }
      None -> {
        return Response {
          status = 401
          headers = singleton("WWW-Authenticate", "Bearer")
          body = toBytes("Unauthorized")
        }
      }
    }
  }
  
  # ============================================
  # ルーティング
  # ============================================
  
  # ルーターの作成
  router : List Route -> Handler
  router routes = { req ->
    findRoute(req, routes) |> match {
      Some(handler) -> { handler(req) }
      None -> { return notFound() }
    }
  }
  
  # ルートのマッチング
  findRoute : Request -> List Route -> Maybe Handler
  findRoute req routes = {
    routes |> findFirst(route -> {
      route.method == req.method && 
      matchPath(route.pattern, req.path)
    }) |> map(route -> { route.handler })
  }
  
  # ============================================
  # ハンドラー例
  # ============================================
  
  # ユーザー一覧を取得
  getUsersHandler : Handler
  getUsersHandler = { req ->
    with database with cache do {
      let cacheKey = { "users:all" }
      
      cached <- get(cacheKey)
      match cached {
        Some(data) -> { return ok(data) }
        None -> {
          users <- query("SELECT * FROM users")
          let response = { json(users) }
          set(cacheKey, response.body, minutes(5))
          return response
        }
      }
    }
  }
  
  # ユーザーを作成
  createUserHandler : Handler
  createUserHandler = { req ->
    with database with logger do {
      let userData = { parseJson(req.body) }
      
      # バリデーション
      validated <- validateUser(userData)
      
      # トランザクション内で作成
      userId <- transaction(do {
        execute("INSERT INTO users (name, email) VALUES (?, ?)", 
                [validated.name, validated.email])
        lastInsertId()
      })
      
      info("Created user: " ++ show(userId))
      
      return json({
        id = userId
        name = validated.name
        email = validated.email
      })
    }
  }
  
  # WebSocketハンドラー
  websocketHandler : Handler
  websocketHandler = { req ->
    with websocket do {
      # WebSocketにアップグレード
      ws <- upgrade(req)
      
      # メッセージループ
      loop(do {
        msg <- receive(ws)
        match msg {
          Text(data) -> {
            # エコーバック
            send(ws, Text("Echo: " ++ data))
          }
          Binary(data) -> {
            send(ws, Binary(data))
          }
          Close -> { break() }
        }
      })
      
      return ok(toBytes("WebSocket closed"))
    }
  }
  
  # ============================================
  # サーバー起動
  # ============================================
  
  startServer : Int -> () ->{Async, IO, Error} ()
  startServer port = do {
    # ルート定義
    let routes = {[
      Route { method = GET;    pattern = "/users";     handler = getUsersHandler },
      Route { method = POST;   pattern = "/users";     handler = createUserHandler },
      Route { method = GET;    pattern = "/users/:id"; handler = getUserHandler },
      Route { method = PUT;    pattern = "/users/:id"; handler = updateUserHandler },
      Route { method = DELETE; pattern = "/users/:id"; handler = deleteUserHandler },
      Route { method = GET;    pattern = "/ws";        handler = websocketHandler },
      Route { method = GET;    pattern = "/health";    handler = { _ -> ok(toBytes("OK")) } }
    ]}
    
    # ミドルウェアを適用
    let app = {
      router(routes)
        |> loggingMiddleware
        |> cacheMiddleware(minutes(10))
        |> errorHandlingMiddleware
    }
    
    # サーバー起動
    with logger {
      info("Starting server on port " ++ show(port))
      
      server <- createServer(app)
      listen(server, port)
      
      info("Server is running at http://localhost:" ++ show(port))
      
      # シグナルハンドリング
      onSignal(SIGINT, do {
        info("Shutting down server...")
        close(server)
        exit(0)
      })
      
      # サーバーを実行
      await(server)
    }
  }
  
  # ============================================
  # エラーハンドリング
  # ============================================
  
  errorHandlingMiddleware : Middleware
  errorHandlingMiddleware handler = { req ->
    with error {
      handler(req)
    } |> match {
      Ok(response) -> { response }
      Err(e) -> {
        with logger {
          error("Request failed: " ++ show(e))
          serverError(show(e))
        }
      }
    }
  }
  
  # ============================================
  # テスト
  # ============================================
  
  test "router matches correct route" = {
    let routes = {[
      Route { method = GET; pattern = "/test"; handler = { _ -> ok(toBytes("test")) } }
    ]}
    
    let req = { Request { 
      method = GET 
      path = "/test"
      headers = empty()
      body = None
      params = empty()
      query = empty()
    }}
    
    let handler = { router(routes) }
    let response = with mock handle { handler(req) }
    
    assert(response.status == 200)
  }
  
  test "middleware chain" = {
    let handler = { req -> ok(toBytes("response")) }
    let logged = ref([])
    
    let testMiddleware = { h -> { req ->
      logged := "before" :: !logged
      res <- h(req)
      logged := "after" :: !logged
      return res
    }}
    
    let wrapped = { handler |> testMiddleware }
    let response = with mock handle { wrapped(mockRequest()) }
    
    assert(!logged == ["after", "before"])
  }
}