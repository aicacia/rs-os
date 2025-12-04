export function blobToUint8Array(blob: Blob): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const arrayBuffer = reader.result;
      if (arrayBuffer instanceof ArrayBuffer) {
        resolve(new Uint8Array(arrayBuffer));
      } else {
        reject(new Error("Blob could not be read as ArrayBuffer"));
      }
    };
    reader.onerror = () => {
      reject(new Error("Error reading Blob"));
    };
    reader.readAsArrayBuffer(blob);
  });
}