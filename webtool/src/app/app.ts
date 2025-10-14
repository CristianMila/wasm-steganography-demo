import { Component, signal, WritableSignal } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { DecimalPipe } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { encodeSecretIntoBmp, decodeSecretFromBmp } from '../bindings/wasm_steganography.js';
import * as FileSaver from 'file-saver';

@Component({
  selector: 'app-root',
  imports: [DecimalPipe, FormsModule],
  standalone: true,
  templateUrl: './app.html',
  styleUrl: './app.css'
})
export class App {
  protected readonly title = signal('Steganography WASM Demo');
  imageBytes: Uint8Array | null = null;
  imageUrl: WritableSignal<string | null> = signal(null);
  imageFileName: string | null = null;
  timeElapsedMs = signal(0);
  decodedSecret: WritableSignal<string | null> = signal(null);
  secretToEncode: string | null = null;

  async onFileSelected(event: Event): Promise<void> {
    const input = event.target as HTMLInputElement;

    if (input.files && input.files.length > 0) {
      const file = input.files[0];
      this.imageFileName = file.name;
      const bytes = await file.arrayBuffer();
      this.imageBytes = new Uint8Array(bytes);
      const imgBlob = new Blob([bytes], { type: 'image/bmp' });

      if (this.imageUrl() !== null) {
        URL.revokeObjectURL(this.imageUrl()!);
      }

      this.imageUrl.set(URL.createObjectURL(imgBlob));
    }
  }

  decode() {
    if (this.imageBytes == null) {
      throw Error("Image not defined");
    }

    try {
      this.measureInMs(() => {
        const secret = decodeSecretFromBmp(this.imageBytes!);
        this.decodedSecret.set(secret);
      }, this.timeElapsedMs);
    } catch (err) {
      console.error(err);
    }
  }

  encode() {
    if (this.imageBytes == null) {
      throw Error("Image not defined");
    }

    if (this.secretToEncode == null) {
      throw Error("secret not defined");
    }

    this.decodedSecret.set(null);

    try {
      const encodedImgBytes = this.measureInMs(() => encodeSecretIntoBmp(this.secretToEncode!, this.imageBytes!), this.timeElapsedMs);
      this.saveToClient(encodedImgBytes, this.imageFileName!);
    } catch(err) {
      console.error(err);
    }

    this.secretToEncode = null;
  }

  measureInMs<T>(fn: () => T, measurementDest: WritableSignal<number>): T {
      const startTime = performance.now();
      const result = fn();
      const stopTime = performance.now();

      measurementDest.set(stopTime - startTime);

      return result;
  }

  saveToClient = (bytes: Uint8Array, filename: string): void => {
    const blob = new Blob([bytes.slice()], { type: 'image/bmp'});
    FileSaver.saveAs(blob, filename);
  }
}
