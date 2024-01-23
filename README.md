# turt

A simple CLI password manager.

The manager has multiple vaults and each vault has multiple entries.

Base idea is that you always have to provide the master password for a transaction.
Motivation is because if you don't either your master password or the encryption
key can be left in memory and be used for decrypting the entire vault.

`turt` can generate a randomized password for you or you can specify an existing
password.

## usage

Note: everything acts on the `default` vault when a vault id isn't specified.

Create an initial vault
```
$ turt create
```

Add an entry to the vault. In this case `super_cool_place` with the username
`myself` and have the password be automatically generated.
```
$ turt add super_cool_place myself
```

Get the credentials for `super_cool_place`.
```
$ turt get super_cool_place
```

In case it is necessary, allows for specifying things like password restrictions
like requiring a digit, symbol, and uppercase character. Often this will happen
just through randomness, but nice to guarantee it.

For a password with a length of 20 that requires a number, symbol, and uppercase
letter:
```
$ turt add with_rules username --pattern "digit+symbol+upper" --length 20
```

## status

Current major todos:

-[] clipboard functionality isn't working on my machine right now
-[] the rust `scrypt` implementation is really slow relative to `openssl`'s implementation
making this annoying to use
-[] running as a server for networked connections

