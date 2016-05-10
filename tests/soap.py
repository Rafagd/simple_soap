from suds.client import Client
url = 'http://localhost:1337/?wsdl'
client = Client(url)

print client
