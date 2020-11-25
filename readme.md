# Me Myself And I

## Ricardo Garza Gerhard - A00816705

### Proyecto de compilador

El compilador está escrito en Rust, usa la librería [Pest](https://github.com/pest-parser/pest) para parsear el lenguaje. En el archivo main se define una función para procesar un archivo de texto con el programa.

### Compilación del ejecutable

Para usarse se necesita tener instalado [Rust](https://rustup.rs/), se usa `cargo build` para generar el programa, que termina en `target/debug/me_myself`. Para correrlo, sólo se tiene que llamar en la terminal `./me_myself`.

#### Comentarios extras sobre la compilación

- Los comentarios se cierran con %%, por problemas con el parser por el newline
- Los comentarios se pueden escribir en casi cualquier parte del programa

### Uso del ejecutable

Para usar el ejecutable, el archivo del lenguaje debe estar en un `.txt`. Primero se debe compilar el programa usando el comando:

```shell
./me_myself compile <input_file> <output_file>
```

Si no se especifica el nombre del archivo de salida, se escribe en el archivo `file.obj`.

Una vez que se tenga el archivo `.obj`, el programa se corre usando el comando:

```shell
./me_myself run <input_file>
```

Si no se especifica el nombre del archivo de entrada, se lee el archivo `file.obj`.

### Ejemplos

En la carpeta de examples se encuentran distintos programas para demostrar el uso del lenguaje. Para correr cualqueir ejemplo se tiene que compilar y correr como cualquier programa de MeMyself:

```shell
./me_myself compile examples/dragon.txt dragon.obj && ./me_myself run dragon.obj
```

## Lenguaje

### Nombre programa

En la primer línea debe ir el nombre del programa:

```
Program testProgram;
```

### Variables globales

A continuación se declaran las variables globales, las cuales pueden ser int, float o char:

```
Program testProgram;
var int: var1, var2; float: var3, var4; char: var5, var6;
```

### Comentarios

Los comentarios son opcionales, y se denota el inicio y final de estos con el `%%`:

```
%% Este es un comentario de MeMyself %%
```

### Operaciones aritméticas

Las operaciones aritméticas permitidas en MeMyself son:
|Operador|Descripción |
|--------|------------|
|+ |Suma |
|- |Resta |
|\* |Multiplicación|
|/ |División |
|% |Módulo |

Estas operaciones se pueden hacer entre ints y floats

### Operaciones booleanas

Las operaciones booleanas permitidas en MeMyself son:
|Operador|Descripción |
|--------|------------|
| < |Menor que|
| > | Mayor que|
| <=|Menor o igual que|
| >=|Mayor o igual que|
|==|Igual a|
|<>|Diferente a|
|&|And
|\||Or

Estas operaciones están permitidas entre:

- Int e int
- Int y float
- Float y float
- Char y Char

### Condicionales

En MyMyself hay 3 tipos de condicionales:

- if..else
- for
- while

```
%% Uso de if %%
if (A <> B) then {

} else {

}

%% Uso de for %%
for (A = 0) to (120) do {

}
%% El for aumenta de uno en uno, mientras el operando del lado izquierdo sea menor que el operando del lado derecho %%

%% Uso de while %%
while (A < 10) do {

}
%% NOTA: el whilke no cambia el valor de la variable a comparar, queda a discreción del programador cambiar dicha variable %%
```

### Escritura

MeMyself permite la escritura de datos a la consola:

```
var int: A;
write(A, "Hello World");
```

### Lectura

MeMyself permite la lectura de datos para ser asignados a variables:

```
var int: A;
read(A);
```

### Funciones

La declaración de funciones se hace empezando en el tipo de retorno de la función, el cual puede ser `int`, `float`, `char`, o `void`, seguido de la palabra `module`, el nombre de la función, y los parámetros, si se necesitan. Después se declaran las variables locales a la función, y luego los estatutos.

- Todo programa debe contar con una función `main` de tipo de retorno `void`, y sin parámetros de entrada, o no compilará el programa.

Un ejemplo sería:

```
int module helloWorld() {
    var char: c; {
        write("Hello World!");
        return(0);
    }
}
```

Para la declaración de retornos en funciones que retornen algún tipo de resultado, se puede agregar el valor de retorno entre los paréntesis:
`return (return_value);`

## Ejemplo completo

Un programa ejemplo completo sería:

```
Program example;
var int: hello;

void module main() {
    var float: test; {
        read(test);
        if (test == 100.0) {
            hello = 200;
        } else {
            hello = 100;
        }
        test = test * hello;
        write(test);
    }
}
```

## Funciones de tortuga

MeMyself es un lenguaje gráfico, por lo que incluye la ya tan conocida "Tortuga" como una salida gráfica. La lista de comandos disponibles para manipular la tortuga son los siguientes:

- `Center()`: regresa la tortuga al centro de la pantalla
- `Forward(float value)`: avanza la tortuga la cantidad de pasos
- `Backward(float value)`: regresa la tortuga la cantidad de pasos
- `Left(float angle)`: mueve la tortuga a la izquierda por cierto ángulo. Valores aceptados: 0-360
- `Right(float angle)`: mueve la tortuga a la derecha por cierto ángulo. Valores aceptados: 0-360
- `PenUp()`: levanta la pluma de la tortuga. Deja de pintar si se mueve
- `PenDown()`: baja la pluma de la tortuga. Empieza a pintar si se mueve
- `Color(float Red, float Green, float Blue)`: cambia el color de la pluma de la tortuga.
- `Size(float)`: cambia el tamaño de la pluma de la tortuga
- `Clear()`: limpia la pantalla de los dibujos
- `Position(float x, float y)`: mueve la tortuga a la posición `x, y`en la pantalla
- `BackgroundColor(float Red, float Green, float Blue)`: cambia el color de fondo de la pantalla
- `FillColor(float Red, float Green, float Blue)`: cambia el color del relleno que se hace en los dibujos.
- `StartFill()`: empieza el relleno del dibujo que se está haciendo. Se tiene que llamar `EndFill()` para empezar otro relleno.
- `EndFill()`: termina el relleno del dibujo que se está haciendo. Se tiene que llamar `StartFill()` antes.
