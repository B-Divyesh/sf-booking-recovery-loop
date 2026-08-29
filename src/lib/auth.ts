import { PublicClientApplication, type AccountInfo } from "@azure/msal-browser";

const tenantId = import.meta.env.VITE_ENTRA_TENANT_ID ?? "35c6fe40-0ec0-46b6-98c6-213ad4de6650";
const subdomain = import.meta.env.VITE_ENTRA_TENANT_SUBDOMAIN ?? "sociobotcustomers";
const clientId = import.meta.env.VITE_ENTRA_CLIENT_ID ?? "25c704f4-465a-47af-80ab-2c489466b697";
const redirectUri = `${window.location.origin}/auth/callback`;

const client = new PublicClientApplication({
  auth: {
    clientId,
    authority: `https://${subdomain}.ciamlogin.com/${tenantId}/`,
    redirectUri,
    knownAuthorities: [`${subdomain}.ciamlogin.com`]
  },
  cache: { cacheLocation: "sessionStorage" }
});

const scopes = ["openid", "profile", "email"];
let initialized = false;

export async function initialiseIdentity(): Promise<AccountInfo | null> {
  if (!initialized) {
    await client.initialize();
    const redirect = await client.handleRedirectPromise();
    if (redirect?.account) client.setActiveAccount(redirect.account);
    initialized = true;
  }
  return client.getActiveAccount() ?? client.getAllAccounts()[0] ?? null;
}

export async function accessToken(): Promise<string | null> {
  const account = await initialiseIdentity();
  if (!account) return null;
  try {
    return (await client.acquireTokenSilent({ account, scopes })).accessToken;
  } catch {
    return null;
  }
}

export async function signIn(): Promise<void> {
  await initialiseIdentity();
  await client.loginRedirect({ scopes });
}

export async function signOut(): Promise<void> {
  const account = await initialiseIdentity();
  if (account) await client.logoutRedirect({ account, postLogoutRedirectUri: window.location.origin });
}

export async function signedInName(): Promise<string | null> {
  const account = await initialiseIdentity();
  return account?.name ?? account?.username ?? null;
}
