# Factory Rules

- A factory is a pure function: `fn(cfg) -> object`. It takes a config struct and returns a parts struct. No side effects. No store access. No asset loading inside the factory body.
- If parts need textures or other runtime handles, the caller resolves them first and passes the resolved values through the cfg, or the factory receives pre-resolved data as plain fields.
- Don't pass `&EStore`, `&Assets`, or any other service into a factory. Don't call `store.add()`, `fire()`, or mutate anything inside a factory method. If you need side effects, that work belongs in the caller.
