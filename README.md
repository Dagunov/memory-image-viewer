# memory-image-viewer

## CLI mode

```
> miv --help

Tool which allows to save image from cv::Mat from memory

Usage: miv.exe [OPTIONS] <PID> <ADDR> <WIDTH> <HEIGHT> <BUF_TYPE>

Arguments:
  <PID>       PID of target process
  <ADDR>      Target memory address in process
  <WIDTH>     Width of image
  <HEIGHT>    Height of image
  <BUF_TYPE>  Buffer type [possible values: CV_8UC1, CV_8UC2, CV_8UC3, CV_8UC4, CV_16UC1, CV_16UC2, CV_16UC3, CV_16UC4, CV_32FC1, CV_32FC2, CV_32FC3, CV_32FC4, CV_64FC1, CV_64FC2, CV_64FC3, CV_64FC4]

Options:
  -o, --out <OUT>  Out file name [default: out]
  -h, --help       Print help
  -V, --version    Print version
```