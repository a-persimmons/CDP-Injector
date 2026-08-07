(() => {
  if (globalThis.cdpHub?.apiVersion === 1) return;

  const definitions = new Map();
  const cleanups = new Map();

  globalThis.cdpHub = {
    apiVersion: 1,
    register(definition) {
      definitions.set(definition.id, definition);
    },
    async activate(id, context) {
      await cleanups.get(id)?.();
      const cleanup = await definitions.get(id)?.activate(context);
      cleanups.set(id, typeof cleanup === "function" ? cleanup : () => {});
    },
    async deactivate(id) {
      await cleanups.get(id)?.();
      cleanups.delete(id);
      definitions.delete(id);
      document
        .querySelectorAll(`[data-cdp-hub-owner="${CSS.escape(id)}"]`)
        .forEach((node) => node.remove());
    },
  };
})();
