/**
 * Trait for an abstract mid-level virtual machine for running the ZMachine. 
 * This trait treats ZMachine constructs (memory, stack, variables, pointers,
 * etc.) as one level above raw bytes. ZMachine opcodes are written in term of
 * the vm::VM trait, allowing simple testing of opcodes without setting up an
 * entire ZMachine memory dump.
 * 
 * PC - the "program counter" for the running machine. Always points to the 
 *   next opcode/operand to be read.
 * Memory - Controlled access to the contents of the zfile. All of the other
 *   ZMachine constructs use this to read/write raw memory from the file. 
 * Stack - A stack for local variables as well as Z-functions to use as temp
 *   scratch space. Also serves as a call stack. 
 * Pointers - Represent offsets into the Memory structure. The ZMachine has
 *   three different treatments for offsets into Memory, each of which may
 *   behave differently on each version of the ZMachine. This attempts to
 *   encapsulate that complexity.
 */
trait FOO {}
