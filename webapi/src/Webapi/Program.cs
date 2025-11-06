using Microsoft.AspNetCore.Mvc;

const string ENV_WASM_MODULE_PATH = "WASM_STEGANOGRAPHY_FILE_PATH";

string[] SUPPORTED_MIME_TYPES = new[] { "image/bmp", "image/jpeg", "image/jpg" };

var builder = WebApplication.CreateBuilder(args);

builder.Configuration.AddEnvironmentVariables();
builder.Services.AddOpenApi();
builder.Services.AddSingleton<SteganographyWasmModule>((serviceProvider) =>
{
    var env = serviceProvider.GetRequiredService<IConfiguration>();
    var webEnv = serviceProvider.GetRequiredService<IWebHostEnvironment>();
    var wasmPath = env.GetValue<string>(ENV_WASM_MODULE_PATH);

    if (!File.Exists(wasmPath))
    {
        throw new FileNotFoundException("Failed locating the WASM modules.");
    }

    return new SteganographyWasmModule(wasmPath);
});
builder.Services.AddMvc();

var app = builder.Build();

if (app.Environment.IsDevelopment())
{
    app.MapOpenApi();
}

app.UseHttpsRedirection();

app.MapPost("/encode", async ([FromBody] EncodeReq req, SteganographyWasmModule steganographyModule) =>
{
    var (encodedImage, fileName) = req.MimeType.ToLowerInvariant() switch {
	"image/bmp" => (await steganographyModule.EncodeIntoBmp(req.Secret, Convert.FromBase64String(req.ImageBase64Encoded)), "encoded_image.bmp"),
	"image/jpeg" or "image/jpg" => (await steganographyModule.EncodeIntoJpeg(req.Secret, Convert.FromBase64String(req.ImageBase64Encoded)), "encoded_image.jpg"),
	_ => throw new InvalidOperationException("Unsupported image mime type. Supported types: image/bmp, image/jpeg, image/jpg.")
    };

    return Results.File(encodedImage, req.MimeType, fileName);
});

app.MapPost("/decode", async (HttpRequest request, SteganographyWasmModule steganographyModule) =>
{
    if (request.ContentType == null)
    {
	return Results.BadRequest("Content type is required.");
    }

    if (!request.HasFormContentType && !SUPPORTED_MIME_TYPES.Contains(request.ContentType))
    {
	return Results.BadRequest($"Unsupported content type: {request.ContentType}. Supported types: image/bmp, image/jpeg, image/jpg.");
    }

    using var memoryStream = new MemoryStream();
    await request.Body.CopyToAsync(memoryStream);
    var imageBytes = memoryStream.ToArray();
    var decodedSecret = request.ContentType.ToLowerInvariant() switch {
	"image/bmp" => await steganographyModule.DecodeFromBmp(imageBytes),
	"image/jpeg" or "image/jpg" => await steganographyModule.DecodeFromJpeg(imageBytes),
	_ => throw new InvalidOperationException("Unsupported image mime type. Supported types: image/bmp, image/jpeg, image/jpg.")
    };

    return Results.Ok(decodedSecret);
}).Accepts<byte[]>("image/bmp", "image/jpeg", "image/jpg");

app.Run();

public record EncodeReq(string Secret, string ImageBase64Encoded, string MimeType);
