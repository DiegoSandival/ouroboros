# Tipos esperados por funcion

Documento corto y explicito para uso diario.

## Referencia rapida

- append(celula: Celula) -> Result<u32>
- read(index: u32) -> Result<Celula>
- read_auth(index: u32, secret: &[u8]) -> Result<Celula>
- update(index: u32, nuevo_genoma: u32, x: u32, y: u32, z: u32) -> Result<()>
- update_auth(index: u32, secret: &[u8], nuevo_genoma: u32, x: u32, y: u32, z: u32) -> Result<()>

## Tipos de cada parametro

- celula: Celula
- index: u32
- secret: &[u8]
- nuevo_genoma: u32
- x: u32
- y: u32
- z: u32

## Como crear una Celula

Celula tiene estos campos:

- hash: [u8; 32]
- salt: [u8; 16]
- genoma: u32
- x: u32
- y: u32
- z: u32

Forma recomendada cuando tienes secret:

- salt: [u8; 16]
- secret: &[u8]
- genoma: u32
- x, y, z: u32
- crear con Celula::with_secret(salt, secret, genoma, x, y, z)

Forma manual:

- hash = Celula::hash_secret(&salt, secret)
- crear con Celula::new(hash, salt, genoma, x, y, z)

## Ejemplo minimo de valores

- index: 0, 1, 2, ... hasta max_records - 1
- secret: b"mi-clave"
- nuevo_genoma: Genoma::LEER_SELF | Genoma::ESCRIBIR_SELF
- x, y, z: 10, 20, 30

## Notas importantes

- index fuera de rango devuelve IndexOutOfBounds.
- secret incorrecto en read_auth/update_auth devuelve Unauthorized.
- append devuelve el slot (u32) donde guardo la celula.
- read devuelve una Celula completa.
- update y update_auth no devuelven datos, solo Ok(()) o error.
- GHOST_FLAG no lo controla el cliente: la DB lo normaliza y lo fija internamente.
