/** @module Interface local:steganography/funcs **/
export function encodeSecretIntoBmp(secret: string, image: Uint8Array): Uint8Array;
export function decodeSecretFromBmp(image: Uint8Array): string;
