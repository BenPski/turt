import websockets
import asyncio
from vault_manager import VaultManager
import ssl
import json
from errors import *

class VaultServer():
    def __init__(self):
        self.vm = VaultManager()
        self.port = self.vm.data['port']

    async def run(self, websocket):
        while True:
            try:
                message = await websocket.recv()
            except websockets.ConnectionClosed:
                print("> Terminated <")
                break
            data = json.loads(message)
            if 'password' in data:
                redacted = data.copy()
                redacted['password'] = "<sensitive>"
                print(f"<<< {redacted}")
            else:
                print(f"<<< {data}")
        
            response = None

            try:
                if data['action'] == 'list':
                    response = self.vm.available()
                elif data['action'] == 'create':
                    entry = data['vault']
                    password = data['password']
                    self.vm.create(entry, password)
                    response = self.vm.available()
                elif data['action'] == 'delete':
                    entry = data['vault']
                    self.vm.delete(entry)
                    response = self.vm.available()
                elif data['action'] == 'vault_list':
                    entry = data['vault']
                    response = self.vm.entries(entry)
                elif data['action'] == 'get_entry':
                    entry = data['vault']
                    label = data['entry']
                    password = data['password']
                    vault = self.vm.get(password, entry)
                    response = vault.get(label)
                elif data['action'] == 'create_entry':
                    entry = data['vault']
                    label = data['entry']
                    password = data['password']
                    info = data['data']
                    vault = self.vm.get(password, entry)
                    vault.create(label, 'generic', info, {}) 
                    vault._save()
                    response = self.vm.entries(entry)
                elif data['action'] == 'update_entry':
                    entry = data['vault']
                    label = data['entry']
                    password = data['password']
                    info = data['data']
                    vault = self.vm.get(password, entry)
                    vault.update(label, info) 
                    vault._save()
                    response = self.vm.entries(entry)

            except IncorrectPassword as e:
                print(e)
                response = "Error"


            if response is not None:
                await websocket.send(json.dumps(response))
                print(f">>> {response}")
            else:
                print(f"> no response <")

async def run_server():
    vs = VaultServer()
    ssl_context = ssl.create_default_context()
    async with websockets.serve(vs.run, 'localhost', vs.port):
        await asyncio.Future()
