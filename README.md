# ouroboros

Base de datos embebida, de archivo fijo, con escritura circular (ring buffer).

## Estado actual (implementacion minima)

- Persistencia en un solo archivo binario.
- Tamano fijo: `max_records * 64` bytes.
- Cada registro es una `Celula` de 64 bytes.
- Escritura circular: cuando llega al final, vuelve al indice 0.
- Modelo SWMR: un solo escritor y multiples lectores.
- El bit `GHOST_FLAG` del genoma se controla internamente para manejar fases del anillo.

## Configuracion

Archivo: `ouroboros.toml`

```toml
data_path = "ouroboros.db"
max_records = 1024
sync_writes = false
```

Reglas:

- `max_records` debe ser mayor a 0.
- `data_path` no puede ser vacio.
- `sync_writes` es opcional (default `false`).
- Si `sync_writes = true`, cada escritura hace `sync_data()` para mayor durabilidad.
- Si `data_path` es relativo, se resuelve respecto a la carpeta del archivo TOML.

## API publica

- `OuroborosDb::open_default()`
- `OuroborosDb::open_from_path(path)`
- `OuroborosDb::open_with_config(&Config)`
- `OuroborosDb::reader()`
- `OuroborosReader::open_default()`
- `OuroborosReader::open_from_path(path)`
- `OuroborosReader::open_with_config(&Config)`
- `append(celula)`
- `read(index)`
- `read_auth(index, secret)`
- `update(index, nuevo_genoma, x, y, z)`
- `update_auth(index, secret, nuevo_genoma, x, y, z)`

Comportamiento de concurrencia:

- Solo se permite un escritor activo por archivo de datos (lock de escritor).
- Se pueden abrir multiples lectores en paralelo.
- Lecturas y escrituras son operaciones separadas; el proyecto no implementa snapshots MVCC.
- Hay pruebas multi-proceso para validar lock de escritor y lectura con escritor activo.

## Modelo de datos

`Celula` ocupa 64 bytes:

- `hash: [u8; 32]`
- `salt: [u8; 16]`
- `genoma: u32` (little-endian en disco)
- `x: u32`
- `y: u32`
- `z: u32`

## Genoma

`Genoma` es un bitmask `u32` con permisos (`LEER_SELF`, `ESCRIBIR_SELF`, etc.).

- Para leer/escribir `GHOST_FLAG`: `get_ghost` y `set_ghost`.
- Para entrada externa: `normalize_genoma` limpia `GHOST_FLAG`.
- Para bytes de cliente: `parse_genoma_le_bytes([u8; 4])` convierte little-endian y limpia `GHOST_FLAG`.

Nota: aunque un cliente envie un genoma con `GHOST_FLAG`, la DB lo reescribe al persistir en `append` y conserva fase en `update`.

## Uso minimo

Ejecutar ejemplo:

```bash
cargo run --example minimal
```

El ejemplo completo esta en `examples/minimal.rs`.


## Limites

- `max_records` es `u32`.
- Tamano teorico maximo del archivo: `(2^32 - 1) * 64` bytes (aprox. 256 GiB).
- El limite real depende de disco y filesystem.
