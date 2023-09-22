TO COM UM PROBLEMA DE SESSION TALVEZ, nao identifiquei ainda, alguma coisa com context? enviar o certificado pelo session, pkcs12 https, 



# nubank-api-rust
Objetivo principal dessa API é criar cobrança PIX e validar se foi ou não pago, foi utilizado o framework Actix-web e SurrealDB.

Endpoints:
1 - Solicitar código do certificado - /certificate/create
método POST
payload:
```json
{
  "login": "SEU CPF",
  "password": "SUA SENHA COM 8 OU MAIS DIGITOS"
}
```
retorno: 
```json
{
	"message": "Success",
	"email": "?****************@gmail.com"
}
```
2 - Validar código do Certificar - /certificate/save (Necessário solicitar o codigo certificado)
metodo POST
payload:
```json
{
  "login": "SEU CPF",
  "code": "CODIGO RECEBIDO NO EMAIL"
}
```
retorno:
Irá retornar code 200.

3 - Criar cobrança PIX - /payment/create (É necessário ter validado o certificado da conta que pretende criar a cobrança pix)
método POST
payload:
```json
{
  "login":  "SEU CPF",
  "amount": 9.00
}
```
retorno:
```json
{
	"id": "92o1265e-365f-45c2-a4f3-50tta66977d7",
	"amount": 9.0,
	"message": null,
	"url": "https://nubank.com.br/pagar/1av2/ZfQuPzLIBY",
	"transactionId": "n8Dr0Bp56Qko",
	"pixAlias": "SEU CPF",
	"brcode": "33439587750002BR.GOV.BCB.PIX20341085062557027322323163142411258.667902BR3465SEU NOME VAI APARECER AQUI6009SAO PAULO71210862733485490601g4Ep9Yu53Ski3205L7BH"
}
```

4 - Validar se PIX foi pago - /payment/details
método GET
payload:
```json
{
  "login": "SEU CPF",
  "id": "ID RETORNADO APÓS CRIAR COBRANÇA PIX"
}
```
retorno: se tiver sido pago irá retornar code 200 com o JSON do pagamento, se não irá retornar Not Found.


Atualmente é utilizado apenas 3 links

1 - gen_certificate é utilizado pra solicitar e validar o certificado. (disc.proxy_list_app_url.gen_certificate).

2 - token é utilizado pra pegar o link ghost_flame_url, o token e o refresh_token. (disc.proxy_list_app_url.token).

3 - ghost_flame_url que é retornado pelo servidor do nubank quando autentica o certificado usando o link token (disc.proxy_list_app_url.token),
e pra ser reutilizado posteriormente ele é salvo no banco. (disc.get_url_ghost_flame()).

Existem vários links não utilizados no código, mas vou deixar para futuras implementações.
(todos os links exceto pelo ghost_flame_url são pegos pela função init() da struct Discovery,  discovery.init().await;)
