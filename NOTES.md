Need to do:
  - call process won't handle returning yet
  - reading result_location always incs PC, but this means we can't store it on the stack during a new_frame
  

# Next
* add more testing to je
* test for storew
* add at least one test for jz
* add at least one test for sub (try hitting all of the sign possibilities)
* get call working correctly and handling arguments
* get ret working
* get ret from a branch working
