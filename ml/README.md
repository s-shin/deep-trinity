# ML

## Modules

* `detris.core`
    * `core.so` should be placed under the `ml` directory.
* `detris.environment`
    * Wrap `detris.core` and provide the interface like OpenAI Gym.
* `detris.simple_a2c`
    * Single thread A2C.
* `detris.zero.*`
    * Implementation based on AlphaZero.
