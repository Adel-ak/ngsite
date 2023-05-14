server {
    listen  80;
    listen  [::]:80;
    server_name porxy.com;

    # security
    include nginxconfig.io/security.conf;

    # logging
    access_log  /var/log/nginx/access.log combined buffer=512k flush=1m;
    error_log   /var/log/nginx/error.log warn;

    # reverse proxy
    location / {
        proxy_pass  http://127.0.0.1:3000;
        proxy_set_header Host $host;
        include nginxconfig.io/proxy.conf;
    }

    # additional config
    include nginxconfig.io/general.conf;
}

# subdomains redirect
server {
    listen  80;
    listen  [::]:80;
    server_name www.porxy.com;
   
    return  301 http://porxy.com$request_uri;
}