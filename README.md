# Raoul

Imperative language done as compilers class final project

## How to run it?

```bash
cargo run -- examples/filename.ra
```

### Flags

- `-d` or `--debug`. Shows debugging message for the developer of the language

# Documentation

## Program structure

- Assignments for global variables
- Function declarations
- Main

## Assigning variables

```go
a = 3;
b = "hello";
c = false;
d = 1.2;
e = [1, 2, 3];
f = read_csv("data.csv");
```

In raoul variable type is infer based on the first value assigned to the variable.
We support the following types:

- int
- float
- string
- boolean
- dataframes
- arrays (only for atomic types, ie. not dataframes)

Due to this freedom, we made the rule of not being able to re-define the type
of a variable. If the variable was initially assigned to a `boolean` type, the
following assignments to this variable must be of type `boolean` or any type
that can be cast into it.

### Global variables

If you wish to make use of global variables there are 2 ways to achieve this:

```go
// Assign before functions
a = 3;

func test(): int {
  return a * b;
}

func main(): void {
  // Use the "global" prefix
  global b = 3;
  print(test());
}
```

It's important to note that using the "global prefix" method sometimes may
cause compilation errors that the "assign before functions" does not. Thus, if
you don't want to worry about a lot about the prior is the recommended method.

## Function declaration

Must be declared before main function. This is a language of good families,
thus there is no polymorphism, meaning that every function's name should be
unique.

```go
func fibonacci(param: int): int {
  ...
  return result;
}
```

## Expressions

```go
x = 2 * (12 + x) >= a;
```

Language supports:

- Arithmetic operations (+, -, \*, /)
- Compare and equality (>, <, >=, <=, ==, !=)
- Logical operations (&&, ||, !)
- Parenthesis for nested expressions

## For-loop declaration

The upper-limit is an inclusive limit. Meaning that if the limit is equals to
`5` the variable of control must be higher than five to exit.

```go
for i = start to limit {
    ...
}
```

## While-loop declaration

```go
i = 0
while(i < 10) {
    ...
    i = i + 1;
}
```

## Conditions declaration

```go
if (cond) {
  ...
} else if (cond_2) {
  ...
} else {
  ...
}
```

## Read from console

Variable assigned the value of input will be of type `string`, nevertheless,
we've made sure that `string` can cast into most of atomic types (with the
exception of boolean). Thus, you can make use of it.

```go
a = input();
```

## Print to console

Its possible to chain multiple string constants and expressions. At the end,
this will always print a new line.

```go
print(var, " ", func());
```

## Dataframe declaration

There can only be one dataframe per program

```go
read_csv("data.csv");
```

## Dataframe shape operations

To get the amount rows and columns of a dataframe you can do the following

```go
rows = get_rows(dataframe);
columns = get_columns(dataframe);
```

## Dataframe operations

Returns the operation value for a given key.

Possible operations:

- Mean: `average()`
- Variance: `variance()`
- Std: `std()`
- Median: `median()`
- Min: `min()`
- Max: `max()`
- Range: `range()`

Arguments:

- 1st argument: must be a dataframe
- 2nd argument: must be a value of type `string`, that represents the column to analyze

```go
avg(data, "key");
```

## Dataframe correlation

Returns correlation value for two columns

```go
correl(data, "key1", "key2");
```

## Plot with dataframe

Scatter plot for two columns in the dataframe, pops up in new window

```go
plot(data, "key1", "key2");
```

> **Note**. Using this command will end the execution of the program, so is
> recommended to be the last one

### Result:

![ScatterPlot](https://imgur.com/0HN1BAH.jpg "Scatter Plot Result")

## Histogram with dataframe

Histogram for a variable in the dataframe, the third argument is the number of bins for the histogram.

Plot pops up in new window.

```go
hist(data, "key1", 10);
```

> **Note**. Using this command will end the execution of the program, so is
> recommended to be the last one

### Result

![Histogram](https://imgur.com/x87d28q.jpg "Histogram Result")

## Main declaration

```go
func main(): void {
    ...
}
```

# Code Examples

There is a bunch a possible valid and invalid files in the
[examples](./src/examples) folder in repo.

# More Documentation

<!-- TODO: Add documentation -->

# Mini Tutorial

<!-- TODO: Add mini tutorial -->
