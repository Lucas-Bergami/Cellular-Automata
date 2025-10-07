# Simulador de Autômatos Celulares

Este projeto implementa um **software interativo de simulação de autômatos celulares**, permitindo a criação, edição e execução de simulações personalizadas sem necessidade de conhecimento prévio em programação.  
O sistema foi desenvolvido como parte do **Trabalho de Conclusão de Curso (TCC)** de **Lucas de Lima Bergami**, com o objetivo de **democratizar o acesso a experimentos e visualizações de autômatos celulares**.

---

## Objetivo do Projeto

A proposta central do software é oferecer uma ferramenta acessível e flexível para **explorar fenômenos complexos emergentes** modelados por autômatos celulares, como propagação de fogo, difusão de doenças, crescimento de populações ou comportamentos físicos simplificados.

O sistema permite:
- Definir **estados personalizados** com cores e pesos;
- Criar **regras de transição** entre estados, com suporte a **probabilidades** e **condições lógicas**;
- Executar e visualizar **simulações dinâmicas**;
- **Importar e exportar** configurações a partir de arquivos de texto.

---

## Exemplo de Arquivo de Configuração

A simulação pode ser configurada a partir de um arquivo `.txt` no seguinte formato:

```txt
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


Esse exemplo representa uma simulação de propagação de fogo em uma floresta, onde:

- Células em “Burning” viram “Empty” com probabilidade 0.5;
- Árvores (“Tree”) adjacentes a “Burning” pegam fogo com probabilidade 1.0;
- Áreas vazias (“Empty”) podem gerar novas árvores com probabilidade 0.1.


## Interface Gráfica com a Biblioteca Iced (v0.12)

O simulador utiliza a biblioteca **[Iced](https://github.com/iced-rs/iced)** (versão 0.12) para a construção da interface gráfica.  
O **Iced** é uma biblioteca moderna para criação de **interfaces gráficas multiplataforma em Rust**, inspirada no **Elm** e com uma arquitetura reativa baseada no padrão **Model–Update–View (MUV)**.

### 🔧 Estrutura Básica

A aplicação segue o padrão clássico de um programa `Iced`:
1. **`struct SimulationState`** – armazena o estado atual da simulação e da interface.  
2. **`Message`** – define os eventos que podem ocorrer na interface (ex: iniciar simulação, carregar arquivo, atualizar célula).  
3. **`Application`** – implementa o comportamento principal da GUI por meio dos métodos:
   - `new()` – inicializa o estado;
   - `update()` – trata as mensagens e atualiza o modelo;
   - `view()` – renderiza os elementos visuais (botões, painéis, canvas, etc.).
  





