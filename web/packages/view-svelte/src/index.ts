import SampleApp from "./SampleApp.svelte";

const app = new SampleApp({
  target: document.querySelector("main")!,
});

export default app;
