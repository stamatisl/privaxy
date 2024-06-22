async function validateCertificate(certPEM, keyPEM) {
    if (!window.crypto || !window.crypto.subtle) {
        console.log("Falling back to server-side validation");
        try {
            const response = await fetch('/api/settings/ca-certificate/validate', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    ca_certificate: certPEM,
                    ca_private_key: keyPEM
                })
            });

            const result = await response.json();
            if (response.ok) {
                return { valid: true };
            } else {
                return { valid: false, error: result.error };
            }
        } catch (error) {
            console.error("Server validation error:", error);
            throw error
        }
    }
    try {
        const certDER = pemToDer(certPEM);
        const keyDER = pemToDer(keyPEM);

        const cert = await window.crypto.subtle.importKey(
            "spki",
            certDER,
            {
                name: "RSASSA-PKCS1-v1_5",
                hash: { name: "SHA-256" },
            },
            true,
            ["verify"]
        );

        const key = await window.crypto.subtle.importKey(
            "pkcs8",
            keyDER,
            {
                name: "RSASSA-PKCS1-v1_5",
                hash: { name: "SHA-256" },
            },
            true,
            ["sign"]
        );

        const keyInfo = await window.crypto.subtle.exportKey("spki", key);
        if (keyInfo.byteLength * 8 < 3072) {
            throw new Error("CA certificate must be at least 3072 bits");
        }

        const testData = new TextEncoder().encode("test data");
        const signature = await window.crypto.subtle.sign("RSASSA-PKCS1-v1_5", key, testData);
        const valid = await window.crypto.subtle.verify("RSASSA-PKCS1-v1_5", cert, signature, testData);

        if (!valid) {
            throw new Error("CA certificate and private key do not match");
        }

        return { valid: true };
    } catch (e) {
        return { valid: false, error: e.message };
    }
}

function pemToDer(pem) {
    const b64 = pem.replace(/-----(BEGIN|END) CERTIFICATE-----/g, "").replace(/\s+/g, "");
    const binary = atob(b64);
    const buffer = new ArrayBuffer(binary.length);
    const view = new Uint8Array(buffer);
    for (let i = 0; i < binary.length; i++) {
        view[i] = binary.charCodeAt(i);
    }
    return buffer;
}

export { validateCertificate };