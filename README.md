# CoD4x launcher
<p align="center">
  <img src="assets/github/banner.png?raw=true" />
</p>

Launcher for the [CoD4x client](https://github.com/callofduty4x/CoD4x_Client_pub), packing an automatic updater.  
The repository also contains the mss32 stub responsible for loading this launcher.

# Build:

## Build prerequisites:
 - [Cargo](https://doc.rust-lang.org/cargo/)
 - [MinGW](http://www.mingw.org/)
 - [NASM](http://www.nasm.us/)

Make sure you've got the necessary binaries in the system `$PATH` environment variable. [Click here to learn more.](https://superuser.com/questions/284342/what-are-path-and-other-environment-variables-and-how-can-i-set-or-use-them)

## Building:
```shell
cargo build --release
```

---

## License:

MIT License

Copyright (c) 2025 CoD4x team

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
