# Star Browser

Star is a lightweight, beginner-friendly desktop browser with built-in AI assistant.


https://github.com/user-attachments/assets/25ede41e-1606-4083-99a6-0ce38ab7a3f9


# Features 
1. AI: can read the page you are on and recall your recent browsing, remembering everything you said.
2. Custom themes/wallpaper: personalize your browser to fit your style.
3. Ad blocker: Block ads wihtout ruining your browsing experience
4. Tab groups: Stay organize with custom colors
5. Media Player: Control your music in your browser
6. Built for privacy and security: Star is local. Your history, favorites and AI memory live on your computer

# AI Memory Demo

https://github.com/user-attachments/assets/9463198b-b413-49e0-9961-20570f49deb9

# Quick Start on how to install this app:
**Notice:**
Star is optimized and tested on **Windows**.
MacOS and Linux builds compile successfully and been tested in virtual machines.
However bacause the virtual machines run slowly, we aren't sure exactly how smooth the performance is yet.
Expect some bugs and problems on these operative systems since they are currently untested on real native hardware.

- [Windows]()
- [MacOs]()
- [Linux]()

The builds aren't code signed so your system will probebaly warn you the first time:

- **Windows:** SmartScreen shows a warning.
Click more info and Run anyway
- **Linux:** make the AppImage executable first with chmod+x.
# Known issues on Linux:
- Web pages inside the browser can render cut off, not filling the whole window
- Popup elements (dropdowns, menus, etc.) can break parts of the UI.
  
# How it works:

we decided to use Tauri framework as our base to build our application.
We choose Tauri to ensure that Star Browser remains lightweight and fast.

## Launch from the Source
You want to build your Browser yourself and add new Features?
You can do it by following these steps!

**Before you start:** You need Node.js (18 or newer) and Rust on every system, plus few things depends on which one you're on

**Windows**
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/), with *Desktop development with C++* selected
- WebView2 — already installed on Windows 11, on Windows 10 get it [here](https://developer.microsoft.com/microsoft-edge/webview2/)
**macOS**
```bash
xcode-select --install
```
 
**Linux (Debian/Ubuntu)**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```
On Fedora use `webkit2gtk4.1-devel`, on Arch use `webkit2gtk-4.1`. It has to be version 4.1, Tauri v2 doesn't work with 4.0.
 
**Setup:**
```bash
# 1. Clone the repository to your computer
git clone https://github.com/Adriii3793/Star-Browser.git
 
# 2. Navigate into the project folder
cd Star-Browser
 
# 3. Install all Node.js dependencies
npm install
 
# 4. Create a .env file in the root folder (check .env.example for the template
#    and variable names) and add your OpenRouter key inside it:
#    OPENROUTER_API_KEY=sk-or-...
#    Get a free key at https://openrouter.ai/keys
#    Make the file with a text editor, not with echo — on Windows that saves it
#    in an encoding that breaks the key.
 
# 5. Launch Star in development mode
npm run tauri dev
 
# 6. Build the app
npm run tauri build
```
 
The first build takes a while because Rust compiles everything from scratch. After that it's much faster. The installers end up in `src-tauri/target/release/bundle/`.
## Credits

- **Adrian** & **[Github](https://github.com/AronMedinaa)** | **[Stardance](https://stardance.hackclub.com/@Aron_3763)**: Aron helped me with this project and we learned new things together.
- **AI Tools**: Helped us learn new programming languages, understand code, optimize, debug, and run tests.


## License

Star Browser is released under the [MIT License](LICENSE) — © 2026 Star Studio.
You are free to use, modify, and distribute it, including commercially, as long as the copyright notice and license text stay with the copies.
