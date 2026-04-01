# FAQ

## Puedo guardar JSON o structs?

Si. La libreria solo almacena bytes. Debes serializar y deserializar por tu cuenta y garantizar que el resultado ocupe exactamente `data_size` bytes.

## Que significa `RecordIndex(0)`?

Es el slot fisico `0` del anillo. No significa "primer evento historico" ni "evento mas reciente".

## Como recupero el ultimo registro escrito?

La API actual no expone un metodo directo para eso. Si solo necesitas saber donde caera el siguiente `append`, usa `next_write_index()`. Si necesitas el ultimo registro cronologico de forma estable, debes modelarlo en tu aplicacion o ampliar la API publica con metadata adicional.

## Que pasa si cambio `OUROBOROS_DATA_SIZE` o `OUROBOROS_MAX_RECORDS`?

Si el `.meta` ya existe, esos valores del entorno se ignoran. Si fuerzas una apertura con una configuracion distinta al tamano real del archivo principal, `open` falla.

## `read` devuelve tambien el `phase bit`?

No. `read` devuelve solo el payload de usuario. El `phase bit` es un detalle interno del motor.

## La libreria maneja concurrencia por si sola?

No. La libreria permite lectura con `&self` y escritura con `&mut self`, pero la coordinacion entre hilos debe hacerla la aplicacion, por ejemplo con `Arc<RwLock<OuroborosDB>>`.

## Que pasa cuando la base da una vuelta completa?

El cursor vuelve a `0`, la fase se alterna y los siguientes `append` empiezan a sobrescribir los slots mas antiguos.

## Como se detecta el punto de recuperacion al reiniciar?

El motor compara bits de fase en el archivo y busca la transicion con busqueda binaria. Por eso la reapertura escala como `O(log N)`.

## Hay soporte para payloads variables?

No en el estado actual del proyecto. Toda la geometria del archivo depende de un tamano fijo por slot.