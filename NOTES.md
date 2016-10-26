Call stack

* Frame pointer - Word - Offset to previous ptr
  * pointer to top of previous frame
  * creates a linked list of frames
  * 0x0000 indicates root frame. (Returning will panic.)
* PC - Word - 