# Linguagem de Definição de Autômatos Celulares

Este documento descreve o formato e as regras da linguagem utilizada para definir autômatos celulares no simulador desenvolvido em Rust.  
Essa linguagem é interpretada diretamente pelo programa e permite descrever tanto os estados quanto as regras de transição de forma simples e legível.

---

## Estrutura geral do arquivo

Um arquivo de definição de autômato deve seguir a estrutura geral:

```
WIDTH <largura> HEIGHT <altura>

STATE {
  <lista de estados>
}

RULES {
  <lista de regras>
}
```

### Exemplo

```
WIDTH 50 HEIGHT 40

STATE {
  Empty(0, 0, 0, 10)
  Tree(0, 200, 0, 7)
  Burning(255, 0, 0, 3)
}

RULES {
  IF current is 'Burning' AND (no conditions) THEN next is 'Empty' WITH PROB 0.5
  IF current is 'Tree' AND count(Burning) >= 1 THEN next is 'Burning' WITH PROB 1.0
  IF current is 'Empty' AND (no conditions) THEN next is 'Tree' WITH PROB 0.1
}
```

---

## Definição do grid

A linha inicial define o tamanho da grade do autômato celular:

```
WIDTH <largura> HEIGHT <altura>
```

**Exemplo:**

```
WIDTH 50 HEIGHT 40
```

Isso cria uma grade com 50 colunas e 40 linhas.

---

## Definição de estados

Os estados são definidos dentro do bloco `STATE { ... }`.  
Cada linha representa um estado com a seguinte sintaxe:

```
Nome(R, G, B, Peso)
```

- **Nome**: identificador do estado (sem espaços).  
- **R, G, B**: valores de cor em RGB (0 a 255).  
- **Peso**: número inteiro que pode ser usado para influenciar regras ou renderização.

**Exemplo:**

```
Tree(0, 200, 0, 7)
Burning(255, 0, 0, 3)
```

Esses estados serão exibidos na interface com as cores e pesos correspondentes.

---

## Definição de regras

As regras de transição são escritas dentro do bloco `RULES { ... }` e seguem a estrutura:

```
IF current is '<estado_atual>' [AND <condições>] THEN next is '<estado_seguinte>' [WITH PROB <probabilidade>]
```

**Componentes:**

- `current`: estado atual da célula.  
- `condições`: expressões opcionais baseadas em vizinhos.  
- `next`: estado resultante após a transição.  
- `PROB`: probabilidade (0.0 a 1.0) de a transição ocorrer.

**Exemplo:**
```
IF current is 'Tree' AND count(Burning) >= 1 THEN next is 'Burning' WITH PROB 1.0
```

---

## Condições

As condições descrevem como os vizinhos afetam o estado da célula. A sintaxe é:

```
count(<nome_estado>) <operador> <valor>
```

**Operadores suportados:**

- `==` Igual a  
- `!=` Diferente de  
- `<` Menor que  
- `<=` Menor ou igual a  
- `>` Maior que  
- `>=` Maior ou igual a  

As condições podem ser combinadas com: `AND`, `OR`, `XOR`

**Exemplo:**
```
IF current is 'Tree' AND count(Burning) >= 1 AND count(Empty) < 3 THEN next is 'Burning' WITH PROB 0.8
```

---

## Probabilidade

O modificador `WITH PROB` define a chance da regra ocorrer.  
Se omitido, a probabilidade padrão é 1.0 (ou seja, 100%).

**Exemplo:**
```
IF current is 'Burning' AND (no conditions) THEN next is 'Empty' WITH PROB 0.5
```

---

## Casos especiais

`(no conditions)`  
Usado quando não há condições de vizinhança.  

**Exemplo:**
```
IF current is 'Empty' AND (no conditions) THEN next is 'Tree' WITH PROB 0.1
```

---

## Exemplo completo

```
WIDTH 50 HEIGHT 40

STATE {
  Empty(0, 0, 0, 10)
  Tree(0, 200, 0, 7)
  Burning(255, 0, 0, 3)
}

RULES {
  IF current is 'Burning' AND (no conditions) THEN next is 'Empty' WITH PROB 0.5
  IF current is 'Tree' AND count(Burning) >= 1 THEN next is 'Burning' WITH PROB 1.0
  IF current is 'Empty' AND (no conditions) THEN next is 'Tree' WITH PROB 0.1
}
```

---

## Como o programa interpreta o arquivo

O programa lê o arquivo linha a linha:

1. Identifica o tamanho da grade (`WIDTH` e `HEIGHT`).  
2. Cria os estados definidos no bloco `STATE { }`.  
3. Analisa as regras dentro de `RULES { }` usando o parser.  

Cada regra é transformada em uma estrutura `TransitionRule` contendo:

- Estado atual  
- Estado seguinte  
- Condições de vizinhança  
- Probabilidade de transição  

Durante a simulação, cada célula do grid:

1. Verifica seu estado atual.  
2. Aplica a primeira regra compatível com as condições.  
3. Calcula se a regra ocorre de acordo com a probabilidade.
