# memory-image-viewer

Memory Image Viewer gives ability to read, display and save images directly
from process memory.\
Memory Image Viewer can be used in CLI or GUI modes (GUI mode can be started
from CLI if no arguments are provided).\
This tool is tested on Windows, Mac and Linux.

## GUI mode

GUI is prefered mode for using memory-image-viewer.\
On Linux, additional requirements (Zenity or Kdialog) may be installed to fully use "Save" feature, but images can still be exported via "Dump"s.
![miv GUI](/assets/interface.png)

## CLI mode

While CLI mode is fully implemented, GUI mode is much, much more feature-rich.\
CLI mode will be supported but it is not planned to update it.

```
> miv --help

Tool which allows to save image from process memory to disk

Usage: miv [OPTIONS] <PID> <ADDR> <WIDTH> <HEIGHT> <BUF_TYPE>

Arguments:
  <PID>       PID of target process
  <ADDR>      Target memory address in process
  <WIDTH>     Width of image
  <HEIGHT>    Height of image
  <BUF_TYPE>  Buffer type [possible values: CV_8UC1, CV_8UC3, CV_8UC4, CV_16UC1, CV_16UC3, CV_16UC4, CV_32FC1, CV_32FC3, CV_32FC4, CV_64FC1, CV_64FC3, CV_64FC4]

Options:
  -o, --out <OUT>  Out file name [default: out]
      --bgr        Use bgr channel order
  -h, --help       Print help
  -V, --version    Print version
```
