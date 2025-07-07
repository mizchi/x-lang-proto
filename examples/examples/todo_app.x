# Todo アプリケーションの例（ブレースファースト構文）

module TodoApp {
  export { Todo, TodoList, addTodo, completeTodo, filterByStatus, todoApp }
  
  # ============================================
  # 型定義
  # ============================================
  
  type TodoId = Int
  
  type Todo = {
    id : TodoId
    title : Text
    description : Text
    completed : Bool
    createdAt : DateTime
  }
  
  type TodoList = List Todo
  
  type Status = {
    All
    Active
    Completed
  }
  
  # ============================================
  # 基本操作
  # ============================================
  
  # 新しいTodoを作成
  createTodo : TodoId -> Text -> Text -> Todo
  createTodo id title desc = {
    Todo {
      id = id
      title = title
      description = desc
      completed = false
      createdAt = now()
    }
  }
  
  # TodoをリストAに追加
  addTodo : Text -> Text -> TodoList ->{State TodoId} TodoList
  addTodo title desc todos = do {
    id <- get()
    put(id + 1)
    let newTodo = { createTodo(id, title, desc) }
    return (newTodo :: todos)
  }
  
  # Todoを完了にする
  completeTodo : TodoId -> TodoList -> TodoList
  completeTodo targetId todos = {
    todos |> map(todo -> {
      if todo.id == targetId {
        todo with { completed = true }
      } else {
        todo
      }
    })
  }
  
  # ステータスでフィルタ
  filterByStatus : Status -> TodoList -> TodoList
  filterByStatus status todos = {
    match status {
      All -> { todos }
      Active -> { todos |> filter(todo -> { not(todo.completed) }) }
      Completed -> { todos |> filter(todo -> { todo.completed }) }
    }
  }
  
  # ============================================
  # UI コンポーネント
  # ============================================
  
  type Component msg = {
    view : () -> Html msg
    update : msg -> () ->{State ComponentState} ()
  }
  
  type Html msg = {
    Text(Text)
    Element(Text, List Attribute, List (Html msg))
    OnClick(msg)
  }
  
  type Attribute = {
    Class(Text)
    Id(Text)
    Style(Text, Text)
  }
  
  # Todo項目の表示
  todoItemView : Todo -> Html TodoMsg
  todoItemView todo = {
    Element("li", [Class(if todo.completed { "completed" } else { "active" })], [
      Element("div", [Class("todo-item")], [
        Element("input", [
          Class("checkbox"),
          OnClick(ToggleComplete(todo.id))
        ], []),
        Element("span", [Class("title")], [Text(todo.title)]),
        Element("p", [Class("description")], [Text(todo.description)])
      ])
    ])
  }
  
  # Todoリストの表示
  todoListView : TodoList -> Status -> Html TodoMsg
  todoListView todos currentStatus = {
    let filtered = { filterByStatus(currentStatus, todos) }
    Element("ul", [Class("todo-list")], 
      filtered |> map(todoItemView)
    )
  }
  
  # ============================================
  # メッセージとアップデート
  # ============================================
  
  type TodoMsg = {
    AddTodo(Text, Text)
    ToggleComplete(TodoId)
    SetFilter(Status)
    ClearCompleted
  }
  
  # メッセージを処理
  updateTodos : TodoMsg -> TodoList ->{State TodoId} TodoList
  updateTodos msg todos = {
    match msg {
      AddTodo(title, desc) -> {
        addTodo(title, desc, todos)
      }
      ToggleComplete(id) -> {
        return completeTodo(id, todos)
      }
      SetFilter(_) -> {
        return todos  # フィルタは表示時に適用
      }
      ClearCompleted -> {
        return (todos |> filter(todo -> { not(todo.completed) }))
      }
    }
  }
  
  # ============================================
  # アプリケーション
  # ============================================
  
  type AppState = {
    todos : TodoList
    filter : Status
    nextId : TodoId
  }
  
  # 初期状態
  initialState : AppState
  initialState = {
    AppState {
      todos = []
      filter = All
      nextId = 1
    }
  }
  
  # メインアプリケーション
  todoApp : () ->{IO, State AppState} ()
  todoApp = do {
    # 初期化
    state <- get()
    
    # イベントループ
    forever(do {
      # UIを描画
      renderUI(state)
      
      # ユーザー入力を待つ
      event <- waitForEvent()
      
      # 状態を更新
      match event {
        UIEvent(msg) -> {
          let newState = with state(state.nextId) {
            let newTodos = { updateTodos(msg, state.todos) }
            let newId = { get() }
            state with { 
              todos = newTodos
              nextId = newId
            }
          }
          put(newState)
        }
        FilterChange(newFilter) -> {
          put(state with { filter = newFilter })
        }
        Quit -> { break() }
      }
    })
  }
  
  # ============================================
  # 永続化
  # ============================================
  
  effect Storage {
    save : Text -> a -> ()
    load : Text -> Maybe a
  }
  
  # Todoリストを保存
  saveTodos : TodoList ->{Storage} ()
  saveTodos todos = {
    save("todos", todos)
  }
  
  # Todoリストを読み込み
  loadTodos : () ->{Storage} TodoList
  loadTodos = {
    match load("todos") {
      Some(todos) -> { todos }
      None -> { [] }
    }
  }
  
  # ============================================
  # テスト
  # ============================================
  
  test "add todo" = with state(1) {
    let todos = { [] }
    let updated = { addTodo("Test", "Description", todos) }
    
    assert(length(updated) == 1)
    assert((head(updated)).title == "Test")
  }
  
  test "complete todo" = {
    let todo = { createTodo(1, "Test", "Desc") }
    let todos = { [todo] }
    let updated = { completeTodo(1, todos) }
    
    assert((head(updated)).completed == true)
  }
  
  test "filter todos" = {
    let todo1 = { createTodo(1, "Active", "Desc") }
    let todo2 = { createTodo(2, "Done", "Desc") with { completed = true } }
    let todos = { [todo1, todo2] }
    
    assert(length(filterByStatus(All, todos)) == 2)
    assert(length(filterByStatus(Active, todos)) == 1)
    assert(length(filterByStatus(Completed, todos)) == 1)
  }
}