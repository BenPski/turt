#!/usr/bin/env python3

"""
The interface should be something like

turt list
<thing1>
 - property 1
 - property2
<thing2>
 - property 1
 - property 2
...

turt list <thing>
<thing>
 - property 1
 - property 2

turt vault <thing>
 retrieves data for thing and outputs to stdout

turt vault <thing> --delete
 deletes the item

turt vault <thing> --set <value>

turt vault <thing> --create <type> --properties prop=val prop=val ...

it it is a password should be able to rotate the password
turt vault <thing> --action rotate
or 
turt vault <thing> --rotate


problems:
would be nice if it wasn't necessary to specify password for listing, but if
any data is stored unencrypted then stuff can be manipulated with simple
filesystem and file manipulation and bork things. A possible way around this
is to keep a checksum or something around that incorporates the files and the
password to check for tampering, but that doesn't prevent anything it is just
a complex solution for checking for bad data that you'd need to restore from
a backup.

seems like it should really just be one file that stores everything. One of the
other oddities of the every individual item approach is encrypted is you can have
a separate password for it which is sort of interesting, but overall weird and
probably a nuisance especially if you typo the password on creation.

current setup prompts for the password pretty late, but probably want it earlier

thinking too far ahead and would be better to just focus on passwords and the
simplest ones with basic assumptions and loosen restriction later
"""

import argparse
import os
import getpass
from vault import Vault
from vault_manager import VaultManager
from server import run_server
import asyncio

def create_password(args):
    if args.symbols is not None:
        symbols = Selection(args.symbols)
    else:
        symbols = Symbol()
    base = Alpha() + Digit() + symbols
    required = []
    if args.include_uppercase:
        required.append(Uppercase())
    if args.include_lowercase:
        required.append(Lowercase())
    if args.include_digit:
        required.append(Digit())
    if args.include_symbol:
        required.append(symbols)

    password = Password(length = args.length, base = base, required = required)
    print(password.generate())


def get(args):
    vault = Vault()
    data = vault.load()
    value = data.get(args.key)
    if value is not None:
        print(value, end='')

def store(args):
    vault = Vault()
    data = vault.load()
    data.update(args.key, args.value)
    vault.save(data)

def create_pass(args):
    vault = Vault()
    data = vault.load()
    properties = {}
    data.create(args.key, 'password', args.value, properties)
    if args.value is None:
        data.grab(args.key).rotate()
    vault.save(data)

def create_cred(args):
    vault = Vault()
    data = vault.load()
    properties = {}
    data.create(args.key, 'credential', {'username': args.username, 'password': args.password}, properties)
    if args.password is None:
        data.grab(args.key).rotate()
    vault.save(data)

def rotate(args):
    vault = Vault()
    data = vault.load()
    entry = data.grab(args.key)
    if entry is not None and entry.data_type in ['password', 'credential']:
        entry.rotate()
    else:
        raise "Trying to rotate a non-password entry doesn't work"
    vault.save(data)

def delete(args):
    vault = Vault()
    data = vault.load()
    data.delete(args.key)
    vault.save(data)

def list_info(args):
    vault = Vault()

    info = vault.load().info()
    for (name, stuff) in info.items():
        print(name, ':')
        for (key, val) in stuff.items():
            print('  ', key, ':', val)

def manager_create(args):
    vm = VaultManager()
    vm.create(args.label)

def manager_list(args):
    vm = VaultManager()
    avail = vm.available()
    for a in avail:
        print(a)

def manager_delete(args):
    vm = VaultManager()
    vm.delete(args.label)

def vault_list(args):
    vm = VaultManager()
    entries = vm.entries(args.label)
    for e in entries:
        print(e)

def vault_get(args):
    vm = VaultManager()
    password = getpass.getpass()
    vault = vm.get(password, args.label)
    if vault is not None:
        print(vault.get(args.entry))

def vault_delete(args):
    vm = VaultManager()
    password = getpass.getpass()
    vault = vm.get(password, args.label)
    if vault is not None:
        vault.delete(args.entry)
        vault._save()

def vault_update(args):
    vm = VaultManager()
    password = getpass.getpass()
    vault = vm.get(password, args.label)
    if vault is not None:
        vault.update(args.entry, args.value)
        vault._save()

def vault_password(args):
    vm = VaultManager()
    password = getpass.getpass()
    vault = vm.get(password, args.label)
    if vault is not None:
        data = vault.create(args.entry, 'password', args.password, {})
        data.rotate()
        vault._save()

def start_server(args):
    asyncio.run(run_server())

def main():
    parser = argparse.ArgumentParser(prog='turt', description='Simple cli password tool')
    parser.set_defaults(func=lambda x: parser.print_help())
    subparsers = parser.add_subparsers()

    server = subparsers.add_parser('server', help='start the server')
    server.set_defaults(func=start_server)

    managerList = subparsers.add_parser('list', help='list available vaults')
    managerList.set_defaults(func=manager_list)

    managerCreate = subparsers.add_parser('create', help='create a vault')
    managerCreate.add_argument('label', help='name of the vault')
    managerCreate.set_defaults(func=manager_create)

    managerDelete = subparsers.add_parser('delete', help='delete a vault')
    managerDelete.add_argument('label', help='name of the vault')
    managerDelete.set_defaults(func=manager_delete)

    vault = subparsers.add_parser('vault', help='interface for dealing with a specific vault')
    vault.set_defaults(func=lambda x: vault.print_help())

    vaultParser = vault.add_subparsers()

    vaultList = vaultParser.add_parser('list', help='list entries in the vault')
    vaultList.add_argument('--label', help='the vault to access')
    vaultList.set_defaults(func=vault_list)

    vaultGet = vaultParser.add_parser('get', help='get an entry in the vault')
    vaultGet.add_argument('--label', help='the vault to access')
    vaultGet.add_argument('entry', help='the entry to lookup')
    vaultGet.set_defaults(func=vault_get)

    vaultDelete = vaultParser.add_parser('delete', help='delete an entry in the vault')
    vaultDelete.add_argument('--label', help='the vault to access')
    vaultDelete.add_argument('entry', help='the entry to delete')
    vaultDelete.set_defaults(func=vault_delete)
    
    vaultUpdate = vaultParser.add_parser('update', help='update an entry in the vault')
    vaultUpdate.add_argument('--label', help='the vault to access')
    vaultUpdate.add_argument('entry', help='the entry to update')
    vaultUpdate.add_argument('value', help='the value to set to')
    vaultUpdate.set_defaults(func=vault_update)

    vaultCreate = vaultParser.add_parser('create', help='create an entry in the vault')
    vaultCreate.set_defaults(func=lambda x: vaultCreate.print_help())

    createParser = vaultCreate.add_subparsers()

    createPassword = createParser.add_parser('password', help='create a password')
    createPassword.add_argument('--label', help='the vault to access')
    createPassword.add_argument('entry', help='the entry to create')
    createPassword.add_argument('password', nargs='?', help='the password to set')
    createPassword.set_defaults(func=vault_password)

    args = parser.parse_args()
    args.func(args)



if __name__ == '__main__':
    main()
