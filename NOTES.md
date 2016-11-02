Need to do:
  - call process won't handle returning yet
  - reading result_location always incs PC, but this means we can't store it on the stack during a new_frame
  

# Next
* add more testing to je
* test for storew
* add at least one test for jz
* add at least one test for sub (try hitting all of the sign possibilities)
* get ret from a branch working


From the ZMachine standards doc (Appendix E)

    Top Ten Opcodes Chart         
    1.   je          195959       Done
    2.   print       142755       Done
    3.   jz          112016       Done
    4.   call_vs     104075       Done (call, since I'm just doing v3)
    5.   print_ret    80870       
    6.   store        71128       
    7.   rtrue        66125       Done
    8.   jump         56534       Done
    9.   new_line     52553       Done
    10.  test_attr    46627       Done

This table is obviously for all machine versions, but it's a reasonable guideline for importance. My current technique has been to implement whatever is crashing a Zork run, but at some point, I'll get to a prompt....

My next ops will probably be:

     1. loadw        Done
     2. jump         Done
     3. put_prop     Done
     4. storew       Done
     5. test_attr    Done
     6. new_line     Done
     7. insert_obj
     8. ret_popped
     9. push
    10. jg
    11. loadb        Done
    12. print        Done
    13. and          Done
    14. print_num    Done
    15. inc_chk      Done
    16. print_char   Done
    17. rtrue        Done
