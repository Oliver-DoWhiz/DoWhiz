# Managing S/MIME Certificates

## Overview

The Gmail S/MIME API enables programmatic management of S/MIME email certificates for users within Google Workspace domains. An administrator must first enable S/MIME for the domain.

The S/MIME standard provides specifications for public key encryption and signing of MIME data. When configured, Gmail:
- Signs outgoing mail using the user's certificate and private key
- Decrypts incoming mail using the user's private key
- Encrypts outgoing mail using the recipient's public key
- Verifies incoming mail using the sender's public key

Individual S/MIME certificates are uploaded via API for specific email aliases, including primary addresses and custom "Send As" addresses. One certificate per alias is designated as default.

## Authorization Methods

Two authorization approaches are available:

1. **Service Account with Domain-Wide Delegation**: Requires enabling domain-wide delegation of authority
2. **OAuth2 Flow**: Requires end-user consent; domain admin must enable "S/MIME API end user access"

## ACL Scopes

The API uses Gmail settings scopes:
- `gmail.settings.basic` - Required for primary SendAs S/MIME updates
- `gmail.settings.sharing` - Required for custom "from" S/MIME updates

## API Operations

The `users.settings.sendAs.smimeInfo` resource provides certificate management methods:

### Upload S/MIME Key

Use `smimeInfo.insert()` to upload a new S/MIME key. Parameters:
- `userId`: User's email address (use "me" for authenticated user)
- `sendAsEmail`: Target alias email address

The request must include `pkcs12` field containing both key and certificate chain. Passwords go in `encryptedKeyPassword`. The API validates:
- Subject matches specified email
- Expiration validity
- Issuing CA is trusted
- Technical compatibility

### List User's S/MIME Keys

`smimeInfo.list()` returns certificates for a given user and alias (PEM format only, excludes private keys).

### Retrieve Specific Keys

`smimeInfo.get()` returns specific S/MIME keys for an alias (PEM format, no private keys).

### Delete S/MIME Key

`smimeInfo.delete()` removes a key using:
- `userId`
- `sendAsEmail`
- `id`: SmimeInfo's immutable ID

**Note**: Deleted keys become unusable. Revoke with issuing authority before deletion.

### Set Default Key

`smimeInfo.setDefault()` marks a key as default for an alias. Only one default per alias exists; calling this unsets the previous default. Gmail associates the key with the latest `not_before` date as default when encrypting outgoing mail.

## Code Examples

### Java: Create SmimeInfo Resource

```java
public static SmimeInfo createSmimeInfo(String filename, String password) {
    SmimeInfo smimeInfo = null;
    InputStream in = null;

    try {
        File file = new File(filename);
        in = new FileInputStream(file);
        byte[] fileContent = new byte[(int) file.length()];
        in.read(fileContent);

        smimeInfo = new SmimeInfo();
        smimeInfo.setPkcs12(Base64.getUrlEncoder().encodeToString(fileContent));
        if (password != null && password.length() > 0) {
            smimeInfo.setEncryptedKeyPassword(password);
        }
    } catch (Exception e) {
        System.out.printf("Error reading certificate file: %s\n", e);
    } finally {
        try {
            if (in != null) in.close();
        } catch (IOException ioe) {
            System.out.printf("Error closing stream: %s\n", ioe);
        }
    }
    return smimeInfo;
}
```

### Python: Create SmimeInfo Resource

```python
import base64

def create_smime_info(cert_filename, cert_password):
    """Create smimeInfo resource for a certificate from file."""
    smime_info = None
    try:
        with open(cert_filename, "rb") as cert:
            smime_info = {}
            data = cert.read()
            smime_info["pkcs12"] = base64.urlsafe_b64encode(data).decode()
            if cert_password and len(cert_password) > 0:
                smime_info["encryptedKeyPassword"] = cert_password
    except (OSError, IOError) as error:
        print(f"Error reading certificate file: {error}")
        smime_info = None

    return smime_info
```

### Java: Upload Certificate

```java
public static SmimeInfo insertSmimeInfo(String userId, String sendAsEmail,
                                        SmimeInfo smimeInfo) throws IOException {
    GoogleCredentials credentials = GoogleCredentials.getApplicationDefault()
        .createScoped(GmailScopes.GMAIL_SETTINGS_SHARING);
    HttpRequestInitializer requestInitializer = new HttpCredentialsAdapter(credentials);

    Gmail service = new Gmail.Builder(new NetHttpTransport(),
        GsonFactory.getDefaultInstance(), requestInitializer)
        .setApplicationName("Gmail samples")
        .build();

    if (sendAsEmail == null) {
        sendAsEmail = userId;
    }

    try {
        SmimeInfo results = service.users().settings().sendAs().smimeInfo()
            .insert(userId, sendAsEmail, smimeInfo)
            .execute();
        System.out.printf("Inserted certificate, id: %s\n", results.getId());
        return results;
    } catch (IOException e) {
        System.err.printf("An error occurred: %s", e);
    }
    return null;
}
```

### Python: Upload Certificate

```python
import google.auth
from googleapiclient.discovery import build
from googleapiclient.errors import HttpError
import create_smime_info

def insert_smime_info():
    """Upload an S/MIME certificate for the user."""
    creds, _ = google.auth.default()

    try:
        service = build("gmail", "v1", credentials=creds)

        user_id = "gduser1@workspacesamples.dev"
        smime_info = create_smime_info.create_smime_info(
            cert_filename="xyz", cert_password="xyz"
        )
        send_as_email = user_id

        results = (service.users().settings().sendAs().smimeInfo()
                  .insert(userId=user_id, sendAsEmail=send_as_email, body=smime_info)
                  .execute())
        print(f'Inserted certificate; id: {results["id"]}')

    except HttpError as error:
        print(f"An error occurred: {error}")
        results = None

    return results
```

## Bulk Operations: CSV Import

For managing certificates across multiple users, use a CSV file structure:

```
user1@example.com,/path/to/user1_cert.p12,cert_password_1
user2@example.com,/path/to/user2_cert.p12,cert_password_2
user3@example.com,/path/to/user3_cert.p12,cert_password_3
```
