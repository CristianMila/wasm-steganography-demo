import { provideZonelessChangeDetection } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { App } from './app';
import { By } from '@angular/platform-browser';

describe('App', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [App],
      providers: [provideZonelessChangeDetection()]
    }).compileComponents();
  });

  it('should create the app', () => {
    const fixture = TestBed.createComponent(App);
    const app = fixture.componentInstance;
    expect(app).toBeTruthy();
  });

  it('should render title', () => {
    const fixture = TestBed.createComponent(App);
    fixture.detectChanges();
    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.querySelector('h2')?.textContent).toContain('STEGANOGRAPHY WASM DEMO');
  });

  it('should decode a previously encoded secret and show it', async () => {
    const fixture = TestBed.createComponent(App);
    const app = fixture.componentInstance;
    const mockIndex = 1;
    const mockIsReq = false;
    const img = await fetch('/base/fixtures/test-encoded.bmp');
    const imgBytes = await img.bytes();
    const mockFile = new File([imgBytes], 'test-encoded.bmp', { type: 'image/bmp' });
    const mockEvt = { target: { files: [mockFile] } };
    spyOn(app, 'decode').and.callThrough();

    // mock uploading a valid image of type "image/bmp"
    await app.onFileSelected(mockEvt as any);
    fixture.detectChanges();
    await fixture.whenStable();

    expect(app.imageBytes).not.toBeNull();

    // find and click the button "Decode"
    const decodeButtonHtml = fixture.debugElement.query(By.css('input[type="button"][value="Decode"]'));
    const decodeButton: HTMLButtonElement = decodeButtonHtml.nativeElement;
    decodeButton.click();

    fixture.detectChanges();
    await fixture.whenStable();
    expect(app.decode).toHaveBeenCalled();

    // check the secret displayed corresponds to what was encoded in the image
    const decodeInputHtml = fixture.debugElement.query(By.css('#decodedSecretInput'));
    const decodeInput: HTMLInputElement = decodeInputHtml.nativeElement;
    expect(decodeInput.value).toBe('foo');
  });

  it('should encode a secret into an image and download it', async () => {
    const fixture = TestBed.createComponent(App);
    const app = fixture.componentInstance;
    const mockIndex = 1;
    const mockIsReq = false;
    const img = await fetch('/base/fixtures/test.bmp');
    const imgBytes = await img.bytes();
    const mockFile = new File([imgBytes], 'test.bmp', { type: 'image/bmp' });
    const mockEvt = { target: { files: [mockFile] } };
    spyOn(app, 'encode').and.callThrough();
    spyOn(app, 'saveToClient').and.stub();

    // mock uploading a valid image of type "image/bmp"
    await app.onFileSelected(mockEvt as any);
    fixture.detectChanges();
    await fixture.whenStable();
    expect(app.imageBytes).not.toBeNull();

    // find and set the secret "foo" to encode
    const secretInput: HTMLInputElement = fixture.debugElement.query(By.css('#encodeSecretInput')).nativeElement;
    secretInput.value = 'foo';
    secretInput.dispatchEvent(new Event('input'));
    fixture.detectChanges();
    await fixture.whenStable();

    // find and click the button "Encode"
    const encodeButtonHtml = fixture.debugElement.query(By.css('input[type="button"][value="Encode"]'));
    const encodeButton: HTMLButtonElement = encodeButtonHtml.nativeElement;
    encodeButton.click();
    fixture.detectChanges();
    await fixture.whenStable();

    // check encoding was performed and downloaded into the clients device
    // with the same bytes as the previously encoded image
    const testImage = await fetch('/base/fixtures/test-encoded.bmp');
    const testImageBytes = await testImage.bytes();
    expect(app.encode).toHaveBeenCalled();
    expect(app.saveToClient).toHaveBeenCalledWith(testImageBytes, 'test.bmp');
  });
});
