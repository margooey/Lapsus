<div align="center">
<img src="./assets/icon.png" width="256" height="256">
<h1>Lapsus</h1>
</div>

<div align="center"><h2>What is this?</h2></div>
Lapsus is an application designed to emulate the feeling of using a trackball. It applies "momentum" to your cursor so that it glides (or slides) across the screen until slowly coming to a stop. Lapsus was born out of Magnes, which was an application designed to emulate the iPadOS cursor as a whole.

<div align="center"><h2>Download</h2></div>

You can download Lapsus on the [Releases](https://github.com/margooey/Lapsus/releases) page. You can also download any built artifacts from the [workflow](https://github.com/margooey/Lapsus/actions). 

You can run Lapsus simply by double-clicking on the app. You can stop the glide behavior by quitting the app via the status bar item or by disabling it via the settings pane.

In the future, Lapsus will come with a .dmg installer to easily drag and drop into your Applications folder. Currently, you will need to run the following command after placing it in your Applications folder to bypass macOS Gatekeeper, as the app is signed ad-hoc:
```
xattr -r -d com.apple.quarantine ~/Applications/Lapsus.app
```

<div align="center"><h2>Build</h2></div>

<h3>CLI (Unsupported, though should still work)</h3>

```shell
cargo build --release
```

<h3>MacOS Bundle (.app)</h3>

```shell
cargo bundle --release
```

<div align="center"><h2>Debugging</h2></div>

Debugging is currently work in progress as there are ongoing changes to support Lapsus's transition into a full MacOS bundle. A lot of the debug logic was ripped out of the code, so there is work to be done to add it back in.

<div align="center"><h2>Environment Variables</h2></div>

`MIN_DT`: The time between each tick in seconds. By default, the cursor will update once every **0.002** seconds (**200Hz**).

`MULTI_FINGER_SUPPRESSION_DEADLINE`: When there is more than one finger on the trackpad, wait this amount of time in seconds to suppress potentially erroneous glides. By default, any glide will be suppressed for **0.15** seconds after the last multi-finger touch.

<div align="center"><h2>Credits</h2></div>

- Yury Korolev: [cidre](https://github.com/yury/cidre)
- jonas-k: [macos-multitouch](https://github.com/jonas-k/macos-multitouch)
- servo: [core-graphics](https://github.com/servo/core-foundation-rs)
- Mads Marquart: [objc2](https://github.com/madsmtm/objc2)


<div align="center"><h2>License</h2></div>
Lapsus is licensed under a custom non-commercial license.
