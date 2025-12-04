export function toArrayBuffer(bytes: Uint8Array)  {
  return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength)
}
