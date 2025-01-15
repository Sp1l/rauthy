import {type I18n} from "./interface.ts";

export const I18nEn: I18n = {
    common: {
        cancel: "Cancel",
        email: "E-Mail",
        errTooShort: "Input too short",
        errTooLong: "Input too long",
        invalidInput: "Invalid Input",
        never: "Never",
        password: "Password",
        required: "Required",
        save: "Save",
    },
    account: {
        account: "User Account",
        accType: "Account Type",
        accTypePasskeyText1: "This account is currently a passkey only account.\nThis means, that you do not have any password, because you don't need one.",
        accTypePasskeyText2: "You can convert your account and add a password. But keep\nin mind, that this implies, that you need to verify each new device with the password additionally.\nYou then cannot just log in on any device, where you have not entered the password beforehand at\nleast once.",
        accTypePasskeyText3: "Do you want to convert your account and add a password?",
        accessExp: "Access Expires",
        accessRenew: "Access Renew Until",
        accessRenewDelete: "Delete the possibility to renew",
        birthdate: "Birthdate",
        city: "City",
        changePassword: "Change Password",
        convertAccount: "Convert Account",
        convertAccountP1: "You can convert your account to a Passkey-Only account.\nThis conversion deletes your password and you can and must only ever login with your registered\npasskeys. Keep in mind, that only passkeys with the additional User Verification will be accepted.\nIf you passkeys support this, you will find a small symbol behind the name of the key on the 'MFA'\npage.",
        country: "Country",
        deviceId: "ID",
        deviceName: "Name",
        devices: "Devices",
        devicesDesc: "Devices linked to this account",
        emailUpdateConfirm: "The E-Mail address has not been updated yet. A message has been\nsent out to your new address. You need to click the confirmation link inside it. Once it has been\nconfirmed, your new address will be updated.",
        emailVerified: "E-Mail verified",
        familyName: "Family Name",
        federatedConvertPassword1: "You have a federated account. This means you log in\nby using an external authentication provider. Your current provider is:",
        federatedConvertPassword2: "You can request a password reset via email. This will\nadd a local password to your account upon completion. You would then be able to log in via your\nexternal provider or by local password. Do you want to request a reset?",
        generateRandom: "Generate Randomly",
        givenName: "Given Name",
        groups: "Groups",
        key: "Key",
        keyUnique: "Key must be unique",
        lastLogin: "Last Login",
        mfaActivated: "MFA activated",
        navInfo: "Info",
        navEdit: "Edit",
        navMfa: "MFA",
        navLogout: "Logout",
        optionalValues: "Optional Values",
        passwordConfirm: "Confirm Password",
        passwordCurr: "Current Password",
        passwordCurrReq: "Current password is required",
        passwordNew: "New Password",
        passwordNewReq: "New password is required",
        passwordNoMatch: "Password verification is required",
        passwordExpiry: "Password expiry",
        passwordPolicyFollow: "You must follow the password policy",
        passwordReset: "Password Reset",
        phone: "Phone",
        providerLink: "Federate Account",
        providerLinkDesc: "You can link this account to one of the following login providers.\nAfter activating this function, you will be redirected to the login page of the chosen one.\nAfter a successful login and if the email matches, your account will be linked.",
        providerUnlink: "Unlink Federation",
        providerUnlinkDesc: "Only if you have set up at least a password or a passkey for this\naccount, you can unlink it from the upstream provider.",
        regDate: "Registration Date",
        regIp: "Registration from IP",
        roles: "Roles",
        street: "Street",
        user: "User",
        userCreated: "User Created",
        userEnabled: "User Enabled",
        userExpiry: "User Expires",
        userVerifiedTooltip: "Secured with fingerprint or PIN",
        validEmail: "Invalid E-Mail format",
        validGivenName: "Your given name should have 1 - 32 non-special characters",
        validFamilyName: "Your family name should have 1 - 32 non-special characters",
        webIdDesc: "You can configure the fields that should be exposed with your WebID.\nThis is a feature used by some networks for decentralized logins. If you do not know what it is,\nyou most probably do not need it.",
        webIdDescData: "You can add custom data fields to your WebID in valid FOAF Vocabulary",
        webIdExpertMode: "Enable Expert Mode",
        zip: "ZIP"
    },
    authorize: {
        clientForceMfa: "This login forces MFA to achieve higher security.\nTo get access, you need to log in to your account and add at least one additional Passkey",
        email: "E-Mail",
        emailBadFormat: "Bad E-Mail format",
        emailRequired: "E-Mail is required",
        emailSentMsg: "If your E-Mail exists, a request has been sent",
        http429: "Too many invalid inputs. Locked until:",
        invalidCredentials: "Invalid credentials",
        invalidKeyUsed: "Invalid Key",
        login: "Login",
        mfaAck: "Acknowledged",
        password: "Password",
        passwordForgotten: "Password forgotten?",
        passwordRequest: "Request",
        passwordRequired: "Password is required",
        provideMfa: "Please login with your MFA device",
        requestExpires: "Request expires",
        signUp: "User Registration"
    },
    device: {
        accept: "Accept",
        autoRedirectAccount: "You will be redirected to your account now",
        closeWindow: "You can close this window now.",
        decline: "Decline",
        desc: "Please enter the {{count}} characters user code from your device.",
        descScopes: "The device requests access to:",
        isAccepted: "The request has been accepted.",
        isDeclined: "The request has been declined.",
        submit: "Submit",
        title: "Device Authorization",
        userCode: "User Code",
        wrongOrExpired: "Wrong or expired code"
    },
    emailChange: {
        title: "E-Mail Change confirmed",
        textChanged: "Your E-Mail address has been changed from",
        textLogin: "You can now log in using your new address.",
        to: "to",
    },
    error: {
        error: "404 Not Found",
        errorText: "The requested data could not be found",
        details: "Show Details",
        detailsText: undefined,
    },
    index: {
        register: "Register",
        accountLogin: "Account",
        adminLogin: "Admin",
    },
    logout: {
        logout: "Logout",
        confirmMsg: "Do you really want to logout and end your session?",
        cancel: "Cancel"
    },
    mfa: {
        p1: "If you plan on using your MFA key with multiple systems like Windows and Android, you should do the registration with Android.",
        p2: "Android is the platform with the least supported features for the passwordless technology. Keys you register with Android work elsewhere too. However, this does not apply the other way around.",
        delete: "Delete",
        errorReg: "Error starting the Registration process",
        invalidKeyUsed: "Invalid Key used",
        lastUsed: "Last used",
        noKey: "No Security key registered on this slot",
        register: "Register",
        registerNew: "Register New Key",
        registerd: "Registered",
        registerdKeys: "Registered Keys",
        passkeyName: "Passkey Name",
        passkeyNameErr: "2 - 32 non-special characters",
        test: "Test",
        testError: "Error starting the Test",
        testSuccess: "Test successful"
    },
    passwordPolicy: {
        passwordPolicy: "Password Policy",
        lengthMin: "Length min",
        lengthMax: "Length max",
        lowercaseMin: "Lowercase letters min",
        uppercaseMin: "Uppercase letters min",
        digitsMin: "Digits min",
        specialMin: "Special characters min",
        notRecent: "Not one of last recent passwords"
    },
    passwordReset: {
        accountLogin: "Account Login",
        badFormat: "Bad Format",
        fidoLink: "https://fidoalliance.org/fido2",
        generate: "Generate",
        newAccDesc1: "You have the option between two account types: passwordless or traditional password",
        newAccDesc2: "The passwordless account is always preferred, because it provides\na way with stronger security. You will need at least one passkey (Yubikey, Apple Touch ID, Windows Hello,\n...) to create such an account. Your device must embrace the FIDO2 standard. For more information\nabout this, you may follow this link: ",
        newAccount: "New Account",
        passwordReset: "Password Reset",
        password: "Password",
        passwordless: "FIDO Passkey",
        passwordConfirm: "Password Confirm",
        passwordNoMatch: "Passwords do not match",
        required: "Required",
        save: "Save",
        success1: "The password has been updated successfully.",
        success2: "You will be redirected shortly.",
        success3: "If you are not being redirected, please click here:",
        successPasskey1: "Your new passkey has been registered successfully.",
        successPasskey2: "Please log into your account and register a second backup key as\nsoon as possible. With a passkey only account, you wil not be able to use a password reset via\nE-Mail in case you lose your current key."
    },
    register: {
        domainAllowed: "Allowed domain:",
        domainErr: "E-Mail domain not allowed",
        domainRestricted: "E-Mail domains are restricted",
        email: "E-Mail",
        emailBadFormat: "Bad E-Mail format",
        emailCheck: "Please check your E-Mail inbox",
        regexName: "Name should have 2 - 32 non-special characters",
        register: "Register",
        success: "Registration successful",
        userReg: "User Registration"
    }
};