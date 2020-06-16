#SysCode

SysCode is my very own programming language (looking awfully like javascript).

For now, if you launch it, it will execute a hard-coded string :
```js
let my_variable = function(a, b, c){
    return a + b + c;
};
let my_variable_3 = my_variable(1, 2, 3);
print(my_variable_3)
```

TODO :

- Add operator overloading.
- Add operator to add members to object.
- Add other types of constant (string, boolean, etc...).
- Add other keywords (if, while, for, etc...).
- Add native interface.