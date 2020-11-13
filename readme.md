# Me Myself And I

## Ricardo Garza Gerhard - A00816705

### Proyecto de compilador

El compilador está escrito en Rust, usa la librería [Pest](https://github.com/pest-parser/pest) para parsear el lenguaje. En el archivo main se define una función para procesar un archivo de texto "memyselfandi.txt", que es un programa de prueba sencillo.

### Compilación

Para usarse se necesita tener instalado [Rust](https://rustup.rs/), se usa `cargo build` para generar el programa, que termina en `target/debug/memyselfandi`. Para correrlo, sólo se tiene que llamar en la terminal `./memyselfandi`. El archivo con el programa debe estar a la misma altura.

#### Comentarios extras

- Los comentarios se cierran con %%, por problemas con el parser por el newline
- Los comentarios se pueden escribir en casi cualquier parte del programa

### Tabla de funciones y variables

Ya se implementan las tablas de funciones y variables. Se usan estructuras y enumeraciones de Rust para manejar todos estos datos. Cuando se corre el programa, la tabla se imprime a la linea de comandos, como `{"main": Func { name: "var_test", ret_type: Void, var_table: [Var { name: "id1", type: "int", value: "" }, Var { name: "id2", type: "int", value: "" }, Var { name: "d3", type: "float", value: "" }] }, "fact": Func { name: "fact", ret_type: Int, var_table: [Var { name: "id1", type: "int", value: "" }, Var { name: "id2", type: "int", value: "" }, Var { name: "d3", type: "float", value: "" }] }}`

### Cuadruplos y condiciones

Se crean los cuadruplos para las operaciones matemáticas, booleanas, escritura, lectura, y condiciones. Los cuadruplos usan registros falsos, con texto manejando los espacios de variables temporales.

Se crean los cuadruplos para funciones no-condicionales (fors)

### Cuadruplos para funciones

Se crean todos los cuadruplos para llamadas de funciones. Está pendiente la verificación completa de tipos semánticos, así como datos extras de funciones (parametros, verificacion de tipos de retorno, etc).

Programa ejemplo:

```
Program var_test;
var int: id1, id2;
float: d3;

int module fact (a);
    var int: A, B, C;
    float: d3, D, F, G;
    {
        for (A = B + 1) to (C + 3) do {
            read(A, C);
            write("A");
        }
        D = F + G;
    }
```

Código generado:

```
Program ID: "var_test"
Processing function fact
{"global": Func { name: "var_test", ret_type: Void, var_table: {"id2": Var { Type: Int, Name: "id2" }, "d3": Var { Type: Float, Name: "d3" }, "id1": Var { Type: Int, Name: "id1" }} }, "fact": Func { name: "fact", ret_type: Int, var_table: {"C": Var { Type: Int, Name: "C" }, "D": Var { Type: Float, Name: "D" }, "F": Var { Type: Float, Name: "F" }, "G": Var { Type: Float, Name: "G" }, "A": Var { Type: Int, Name: "A" }, "B": Var { Type: Int, Name: "B" }, "d3": Var { Type: Float, Name: "d3" }} }}
0 Sum Some(Var { Type: Int, Name: "1" }) Some(Var { Type: Int, Name: "B" }) Var { Type: Int, Name: "t0" }
1 Assign Some(Var { Type: Int, Name: "t0" }) None Var { Type: Int, Name: "A" }
2 Assign Some(Var { Type: Int, Name: "A" }) None Var { Type: Int, Name: "VC" }
3 Sum Some(Var { Type: Int, Name: "3" }) Some(Var { Type: Int, Name: "C" }) Var { Type: Int, Name: "t0" }
4 Assign Some(Var { Type: Int, Name: "t0" }) None Var { Type: Int, Name: "VF" }
5 LessThan Some(Var { Type: Int, Name: "VC" }) Some(Var { Type: Int, Name: "VF" }) Var { Type: Int, Name: "t0" }
6 GotoF None None Var { Type: Int, Name: "13" }
7 Read None None Var { Type: Int, Name: "A" }
8 Read None None Var { Type: Int, Name: "C" }
9 Print None None Var { Type: Int, Name: "\"A\"" }
10 Sum Some(Var { Type: Int, Name: "VF" }) Some(Var { Type: Int, Name: "1" }) Var { Type: Int, Name: "VC" }
11 Assign Some(Var { Type: Int, Name: "VC" }) None Var { Type: Int, Name: "A" }
12 Goto None None Var { Type: Int, Name: "5" }
13 Sum Some(Var { Type: Float, Name: "G" }) Some(Var { Type: Float, Name: "F" }) Var { Type: Float, Name: "t0" }
14 Assign Some(Var { Type: Float, Name: "t0" }) None Var { Type: Float, Name: "D" }
```

### Mapa de Memoria de ejecución

El sistema genera el código .obj después de procesar el lenguaje. El archivo queda de la forma:
```
C 0 30000 Int
C 13 30001 Int
C 1 30002 Int
G 1 1 0
F main 2 4 0 0 1 0 0 1
A Era -1 -1 main
A Goto -1 -1 2
A Assign 30000 -1 10001
A Assign 30001 -1 10002
A LessThan 10001 10002 23000
A GotoF 23000 -1 10
A Print -1 -1 10001
A Sum 10001 30002 20000
A Assign 20000 -1 10001
A Goto -1 -1 4
A EndFunc -1 -1 
```
La primer letra indica el tipo de dato que se va a procesar:
* C: Constante, valor, posición, tipo de variable
* G: Global, conteo ints, conteo floats, conteo chars
* F: función, cuadruplo inicial, local ints, local floats, local chars, temp ints, temps floats, temp chars, temp bools
* A: cuádruplo, acción, operador izquierdo, operador derecho, operador final

En la generación del código para la máquina virtual se procesa el tipo de dato que se va a accesar, dependiendo de la acción. Ya funcionan todos los códigos de operación, menos la llamada de funciones y el for.

Código ejemplo:
```
Program var_test;
var int: H;
float: G;

void module main (int F) {
    var int: A, B, C;
    {
        A = 0;
        B = 13;
        while (A < B) do {
            write(A);
            A = A + 1;
        }
    }
}
```

Salida  de máquina virtual:
```
Int(0)
Int(1)
Int(2)
Int(3)
Int(4)
Int(5)
Int(6)
Int(7)
Int(8)
Int(9)
Int(10)
Int(11)
Int(12)
Memory { global: {5000: Int(0), 6000: Float(0.0)}, local: {10000: Int(0), 10001: Int(13), 10003: Int(0), 10002: Int(13)}, temp: {20000: Int(13), 23000: Bool(false)}, cte: {30001: Int(13), 30000: Int(0), 30002: Int(1)}, local_stack: [], temp_stack: [] }
```
