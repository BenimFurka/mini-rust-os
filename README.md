# Mini Rust OS

[![Rust](https://img.shields.io/badge/Rust-nightly-blue?logo=rust)](https://www.rust-lang.org/) 
[![Build with Make](https://img.shields.io/badge/build-make-green)](https://www.gnu.org/software/make/) 
[![License](https://img.shields.io/badge/License-LGPL%20v3-blue.svg)](LICENSE)

**Mini Rust OS** — минималистичная операционная система на Rust для архитектуры x86_64. Проект показывает основы низкоуровневой работы с железом, загрузку в 64-битном режиме, простую командную строку и базовые драйверы устройств.

## Быстрый старт

### Требования
- Rust nightly
- NASM
- QEMU
- grub-mkrescue
- make

### Сборка и запуск

```sh
make        # Сборка iso файла
make run    # Сборка и запуск в QEMU с виртуальным диском
```

### Очистка
```sh
make clean
```

## Возможности

- Загрузка в 64-битный режим через собственный загрузчик (NASM)
- Простая командная строка с базовыми командами для тестирования железа
- Работа с VGA-буфером для вывода текста
- Модульная архитектура: драйверы, команды, обработка команд, вывод
- Пример для обучения и экспериментов с Rust в no-std

## Благодарности

Загрузка ядра на ассемблере основана на проекте [justin-uy/simple-rust-os](https://github.com/justin-uy/simple-rust-os).

## Лицензия

Этот проект распространяется под лицензией [LGPL v3](LICENSE).
