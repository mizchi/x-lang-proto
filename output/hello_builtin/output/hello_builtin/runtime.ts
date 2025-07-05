// x Language Runtime for TypeScript

// Effect System Runtime
export class EffectContext {
  private handlers = new Map<string, Function>();
  private stack: any[] = [];

  addHandler(effect: string, handler: Function): void {
    this.handlers.set(effect, handler);
  }

  perform<T>(effect: string, operation: string, ...args: any[]): T {
    const handler = this.handlers.get(effect);
    if (!handler) {
      throw new Error(`Unhandled effect: ${effect}.${operation}`);
    }
    return handler(operation, ...args);
  }
}

// Utility Functions
export function curry<T extends (...args: any[]) => any>(fn: T): any {
  return function curried(...args: any[]): any {
    if (args.length >= fn.length) {
      return fn.apply(this, args);
    } else {
      return function (...args2: any[]) {
        return curried.apply(this, args.concat(args2));
      };
    }
  };
}

export class MatchError extends Error {
  constructor(value: any) {
    super(`Non-exhaustive pattern match for value: ${JSON.stringify(value)}`);
  }
}
