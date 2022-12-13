Vagrant.configure("2") do |config|

  BOX_IMAGE = "generic/arch"
  BASE_NETWORK = "192.168.56"   # For HostOnly
  # BASE_NETWORK = "192.168.1"  # For intent
  
  PROXY_HTTP = "http://10.0.2.2:7777"
  PROXY_HTTPS = "http://10.0.2.2:7777"
  PROXY_EXCLUDE = "localhost,127.0.0.1"

  DB_USER = "worker"
  DB_USER_PASS = "root"
  DB_NAME = "service"

  MB_USER = "worker"
  MB_USER_PASS = "root"
  MB_VHOST_NAME = "vhost"
  MB_USER_HOST_PERM = "\".*\" \".*\" \".*\""

  SSH_INSERT_KEY = true
  PROXY_ENABLED = false

  RABBIT_COOKIE = "LSOEHRZCUPQIRNYDHUSL"

  DB_SERVER_IP = "#{BASE_NETWORK}.10"
  MB_SERVER_IP1 = "#{BASE_NETWORK}.11" # Main server
  MB_SERVER_IP2 = "#{BASE_NETWORK}.12"
  MB_SERVER_IP3 = "#{BASE_NETWORK}.13"
  LB_SERVER_IP = "#{BASE_NETWORK}.14"

  WEB_SERVER_IP1 = "#{BASE_NETWORK}.21"
  WEB_SERVER_IP2 = "#{BASE_NETWORK}.22"
  WEB_SERVER_IP3 = "#{BASE_NETWORK}.23"

  config.vm.define "db" do |subconfig|
    subconfig.vm.box = BOX_IMAGE

    subconfig.vm.network :private_network,
      ip: "#{DB_SERVER_IP}",
      adapter: 2                  # HostOnly
      # virtualbox__intnet: true  # Intnet

    subconfig.vm.hostname = "server.db"
    subconfig.ssh.insert_key = SSH_INSERT_KEY

    subconfig.vm.provider "virtualbox" do |vb|
      vb.name = "DBServer"
      vb.memory = "1024"
      vb.cpus = 1
    end

    if Vagrant.has_plugin?("vagrant-proxyconf") && PROXY_ENABLED
      subconfig.proxy.http = PROXY_HTTP
      subconfig.proxy.https = PROXY_HTTPS
      subconfig.proxy.no_proxy = PROXY_EXCLUDE
    end
    
    $setup = <<-EOF
    sed -i '/SigLevel    = Required DatabaseOptional/c\\SigLevel = Never' /etc/pacman.conf
    pacman -S archlinux-keyring openssl --noconfirm

    pacman -Syu --noconfirm
    pacman -S mariadb --noconfirm
    mariadb-install-db --user=mysql --basedir=/usr --datadir=/var/lib/mysql
    systemctl enable mariadb
    systemctl start mariadb
    mysql -e "CREATE USER IF NOT EXISTS '#{DB_USER}'@'%' IDENTIFIED BY '#{DB_USER_PASS}';"
    mysql -e "CREATE USER IF NOT EXISTS '#{DB_USER}'@'localhost' IDENTIFIED BY '#{DB_USER_PASS}';"
    mysql -e "FLUSH PRIVILEGES;"
    mysql -e "CREATE DATABASE IF NOT EXISTS #{DB_NAME};"
    mysql -e "GRANT ALL PRIVILEGES ON #{DB_NAME}.* TO '#{DB_USER}'@'%';"
    mysql -e "GRANT ALL PRIVILEGES ON #{DB_NAME}.* TO '#{DB_USER}'@'localhost';"
    
    date > /etc/vagrant_provisioned_at
    EOF

    subconfig.vm.provision "shell", inline: $setup
  end

  config.vm.define "mb1" do |subconfig|
    subconfig.vm.box = BOX_IMAGE

    subconfig.vm.network :private_network,
      ip: "#{MB_SERVER_IP1}",
      adapter: 2                  # HostOnly
      # virtualbox__intnet: true  # Intnet

    subconfig.vm.network "forwarded_port", guest: 15672, host: 15672

    subconfig.vm.hostname = "server.mb"
    subconfig.ssh.insert_key = SSH_INSERT_KEY

    subconfig.vm.provider "virtualbox" do |vb|
      vb.name = "MBServer1"
      vb.memory = "1024"
      vb.cpus = 1
    end

    if Vagrant.has_plugin?("vagrant-proxyconf") && PROXY_ENABLED
      subconfig.proxy.http = PROXY_HTTP
      subconfig.proxy.https = PROXY_HTTPS
      subconfig.proxy.no_proxy = PROXY_EXCLUDE
    end
    
    $setup = <<-EOF
    sed -i '/SigLevel    = Required DatabaseOptional/c\\SigLevel = Never' /etc/pacman.conf
    pacman -S archlinux-keyring openssl --noconfirm

    pacman -Syu --noconfirm
    pacman -S erlang rabbitmq --noconfirm
    
    # /etc/rabbitmq/rabbitmq-env.conf
    # NODENAME=rabbit@node01
    # NO_IP_ADDRESS=0.0.0.0
    # NODE_PORT=5672
    # cluster_formation.peer_discovery_backend = classic_config
    # cluster_formation.classic_config.nodes.1 = rabbit@node02
    # cluster_formation.classic_config.nodes.2 = rabbit@node03

    # This is the first node
    
    systemctl enable rabbitmq
    systemctl start rabbitmq

    rabbitmq-plugins enable rabbitmq_management
    
    systemctl stop rabbitmq

    # Hostnames
    sed -i "/NODENAME=/c\\NODENAME=rabbit@server1" /etc/rabbitmq/rabbitmq-env.conf
    echo "#{MB_SERVER_IP1} server1" | tee -a /etc/hosts
    echo "#{MB_SERVER_IP2} server2" | tee -a /etc/hosts
    echo "#{MB_SERVER_IP3} server3" | tee -a /etc/hosts

    # Set cookie
    echo "#{RABBIT_COOKIE}" | tee /var/lib/rabbitmq/.erlang.cookie

    systemctl start rabbitmq

    rabbitmqctl add_user #{MB_USER} #{MB_USER_PASS}
    rabbitmqctl add_vhost #{MB_VHOST_NAME}
    rabbitmqctl set_permissions -p #{MB_VHOST_NAME} #{MB_USER} #{MB_USER_HOST_PERM}
    rabbitmqctl set_user_tags #{MB_USER} administrator

    date > /etc/vagrant_provisioned_at
    EOF

    subconfig.vm.provision "shell", inline: $setup
  end

  config.vm.define "mb2" do |subconfig|
    subconfig.vm.box = BOX_IMAGE

    subconfig.vm.network :private_network,
      ip: "#{MB_SERVER_IP2}",
      adapter: 2                  # HostOnly
      # virtualbox__intnet: true  # Intnet

    subconfig.vm.hostname = "server.mb"
    subconfig.ssh.insert_key = SSH_INSERT_KEY

    subconfig.vm.provider "virtualbox" do |vb|
      vb.name = "MBServer2"
      vb.memory = "1024"
      vb.cpus = 1
    end

    if Vagrant.has_plugin?("vagrant-proxyconf") && PROXY_ENABLED
      subconfig.proxy.http = PROXY_HTTP
      subconfig.proxy.https = PROXY_HTTPS
      subconfig.proxy.no_proxy = PROXY_EXCLUDE
    end
    
    $setup = <<-EOF
    sed -i '/SigLevel    = Required DatabaseOptional/c\\SigLevel = Never' /etc/pacman.conf
    pacman -S archlinux-keyring openssl --noconfirm

    pacman -Syu --noconfirm
    pacman -S erlang rabbitmq --noconfirm

    systemctl enable rabbitmq
    systemctl start rabbitmq

    rabbitmq-plugins enable rabbitmq_management

    # Hostnames
    sed -i "/NODENAME=/c\\NODENAME=rabbit@server2" /etc/rabbitmq/rabbitmq-env.conf
    echo "#{MB_SERVER_IP1} server1" | tee -a /etc/hosts
    echo "#{MB_SERVER_IP2} server2" | tee -a /etc/hosts
    echo "#{MB_SERVER_IP3} server3" | tee -a /etc/hosts
    
    # Set cookie
    echo "#{RABBIT_COOKIE}" | tee /var/lib/rabbitmq/.erlang.cookie

    systemctl restart rabbitmq

    rabbitmqctl stop_app
    rabbitmqctl reset
    rabbitmqctl join_cluster rabbit@server1
    rabbitmqctl start_app

    date > /etc/vagrant_provisioned_at
    EOF

    subconfig.vm.provision "shell", inline: $setup
  end

  config.vm.define "mb3" do |subconfig|
    subconfig.vm.box = BOX_IMAGE

    subconfig.vm.network :private_network,
      ip: "#{MB_SERVER_IP3}",
      adapter: 2                  # HostOnly
      # virtualbox__intnet: true  # Intnet

    subconfig.vm.hostname = "server.mb"
    subconfig.ssh.insert_key = SSH_INSERT_KEY

    subconfig.vm.provider "virtualbox" do |vb|
      vb.name = "MBServer3"
      vb.memory = "1024"
      vb.cpus = 1
    end

    if Vagrant.has_plugin?("vagrant-proxyconf") && PROXY_ENABLED
      subconfig.proxy.http = PROXY_HTTP
      subconfig.proxy.https = PROXY_HTTPS
      subconfig.proxy.no_proxy = PROXY_EXCLUDE
    end
    
    $setup = <<-EOF
    sed -i '/SigLevel    = Required DatabaseOptional/c\\SigLevel = Never' /etc/pacman.conf
    pacman -S archlinux-keyring openssl --noconfirm

    pacman -Syu --noconfirm
    pacman -S erlang rabbitmq --noconfirm

    systemctl enable rabbitmq
    systemctl start rabbitmq

    rabbitmq-plugins enable rabbitmq_management

    # Hostnames
    sed -i "/NODENAME=/c\\NODENAME=rabbit@server3" /etc/rabbitmq/rabbitmq-env.conf
    echo "#{MB_SERVER_IP1} server1" | tee -a /etc/hosts
    echo "#{MB_SERVER_IP2} server2" | tee -a /etc/hosts
    echo "#{MB_SERVER_IP3} server3" | tee -a /etc/hosts

    # Set cookie
    echo "#{RABBIT_COOKIE}" | tee /var/lib/rabbitmq/.erlang.cookie

    systemctl restart rabbitmq

    rabbitmqctl stop_app
    rabbitmqctl reset
    rabbitmqctl join_cluster rabbit@server1
    rabbitmqctl start_app

    date > /etc/vagrant_provisioned_at
    EOF

    subconfig.vm.provision "shell", inline: $setup
  end

  config.vm.define "lb" do |subconfig|
    subconfig.vm.box = BOX_IMAGE

    subconfig.vm.network :private_network,
      ip: "#{LB_SERVER_IP}",
      adapter: 2                  # HostOnly
      # virtualbox__intnet: true  # Intnet

    subconfig.vm.hostname = "server.lb"
    subconfig.ssh.insert_key = SSH_INSERT_KEY

    subconfig.vm.provider "virtualbox" do |vb|
      vb.name = "LBServer"
      vb.memory = "1024"
      vb.cpus = 1
    end

    if Vagrant.has_plugin?("vagrant-proxyconf") && PROXY_ENABLED
      subconfig.proxy.http = PROXY_HTTP
      subconfig.proxy.https = PROXY_HTTPS
      subconfig.proxy.no_proxy = PROXY_EXCLUDE
    end
    
    CONF =
    <<-EOF
    events {}

    http {
      upstream backend {
        server #{WEB_SERVER_IP1};
        server #{WEB_SERVER_IP2};
        server #{WEB_SERVER_IP3};
      }

      server {
        listen 80;
        server_name = prototype.com;

        location / {
          proxy_pass http://backend;
        }
      }
    }
    EOF

    $setup = <<-EOF
    sed -i '/SigLevel    = Required DatabaseOptional/c\\SigLevel = Never' /etc/pacman.conf
    pacman -S archlinux-keyring openssl --noconfirm

    pacman -Syu --noconfirm
    pacman -S nginx --noconfirm

    systemctl enable nginx
    
    echo "#{CONF}" > /etc/nginx/nginx.conf

    date > /etc/vagrant_provisioned_at
    EOF

    subconfig.vm.provision "shell", inline: $setup
  end

end