/*
 * What a new repository's identity is, worked out from the flags. Pure: values in, values out.
 */

/** What `identityFor` reads from the flags. */
export interface IdentityFlags {
  readonly [flag: string]: string | true | undefined;
  readonly org?: string | true;
  readonly repo?: string | true;
  readonly serverId?: string | true;
  readonly title?: string | true;
  readonly summary?: string | true;
}

/** A new repository's identity, every field filled. */
export interface Identity {
  readonly org: string;
  readonly repo: string;
  readonly serverId: string;
  readonly title: string;
  readonly summary: string;
}

/** A flag's value as text, or `undefined` when it is missing or given with no value. */
function text(value: string | true | undefined): string | undefined {
  return typeof value === "string" ? value : undefined;
}

/** `my-repo` is `My Repo`. */
function titleCase(name: string): string {
  return name
    .split(/[^A-Za-z0-9]+/)
    .filter(Boolean)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}

/**
 * The full identity for `org`/`repo`. `serverId`, `title` and `summary` default from them.
 *
 * `serverId` defaults to the first word of the org and the repo: `LXAgents-Skills` and
 * `security` give `lxagents-security`, the way the existing servers are named.
 */
export function identityFor(flags: IdentityFlags): Identity {
  const [org, repo, serverId, title, summary] = [flags.org, flags.repo, flags.serverId, flags.title, flags.summary].map(text);
  if (!org || !repo) throw new Error("Both --org and --repo are required.");
  if (!/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(org) || !/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(repo)) {
    throw new Error("--org and --repo take GitHub names: letters, digits, '.', '_' and '-'.");
  }

  const brand = org.split(/[^A-Za-z0-9]+/)[0] ?? org;
  return {
    org,
    repo,
    serverId: serverId ?? `${brand}-${repo}`.toLowerCase(),
    title: title ?? `${brand} ${titleCase(repo)}`,
    summary: summary ?? `The ${repo} instruction set, served read-only.`,
  };
}
