# turt

A simple manager for vaults that encrypt your data. Every vault has a master
password and a series of entries that stores some data.

The goal of this was to think about how to approach the situation of someone
getting access to your computer or device because you left it unlocked or something.
In most cases a password manager or something that stores secure data usually 
leaves you logged in or stoes the data in memory so the other person could
very easily get this data if they wanted to. Well, to get around this you always
prompt for the password and have both the master password and the derived
encryption key be cleared from memory and that is what I was attempting here.

In reality, if someone is getting access to your computer or device and they
want to do something malicious you are pretty hosed anyways. This has potential
to be a step up in security, but it is less convenient on the user side since
it requires that the passwords never be remembered.

## status

Right now this is a prototype and is primarily intended to be used with the
accompanying [browser extension](https://github.com/BenPski/turt-extension).

Can use the cli as a CRUD application, but it isn't too fully featured.

Has some workings for things like password rotations or configuring rules about
generating passwords.

