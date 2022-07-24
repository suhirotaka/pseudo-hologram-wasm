# pseudo-hologram-wasm
A pseudo hologram generator in WASM.

### Install
`npm install pseudo-hologram-wasm`

### Example
1. Add `video` and `canvas` tag in HTML.
    - `video` plays your original video to be converted to hologram.
    - Hologram will be rendered in `canvas`.
    ```
    <h4>Original Video:</h4>
    <video id="orig-video" controls="true" crossorigin="anonymous" width="400" height="400">
      <source src="https://upload.wikimedia.org/wikipedia/commons/6/6c/Polar_orbit.ogv" type="video/ogg">
    </video>
    <hr />
    <h4>Hologram:</h4>
    <canvas id="hologram-canvas"></canvas>
    ```

2. Import `pseudo-hologram-wasm` in JS and call `PseudoHologram.draw` with canvas/video tag IDs.
    ```
    import * as hologram from "pseudo-hologram-wasm";
    
    let intervalHandle = hologram.PseudoHologram.draw('hologram-canvas', 'orig-video');
    
    // You can stop rendering hologram as following.
    // setTimeout(() => clearInterval(intervalHandle['interval_id']), 3000)
    ```
