## Very simple Rust M4 demonstration.  

### Running project:  
     ```  
     cargo run  
     ```  

### Expected result:  

```
===============================  
Input macro:   
  
define(`foo', `Hello world.')  
foo  
  
Output result:  
  
Hello world.  
  
===============================  
Input macro:  
  
define(`exch', `$2, $1')  
exch(arg1, arg2)  
  
Output result:  
  
arg2, arg1  
```