Vagrant.configure("2") do |config|

  BOX_IMAGE = "generic/arch"
  BASE_NETWORK = "192.168.56"
  
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

  DB_SERVER_IP = "#{BASE_NETWORK}.10"
  MB_SERVER_IP = "#{BASE_NETWORK}.11"
  WEB_SERVER_IP = "#{BASE_NETWORK}.12"

  config.vm.define "db" do |subconfig|
    subconfig.vm.box = BOX_IMAGE

    subconfig.vm.network :private_network,
      ip: "#{DB_SERVER_IP}",
      adapter: 2

    subconfig.vm.hostname = "server.db"
    subconfig.ssh.insert_key = SSH_INSERT_KEY

    subconfig.vm.provider "virtualbox" do |vb|
      vb.name = "DBServerVM"
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

    config.vm.provision "shell", inline: $setup
  end

  config.vm.define "mb" do |subconfig|
    subconfig.vm.box = BOX_IMAGE

    subconfig.vm.network :private_network,
      ip: "#{MB_SERVER_IP}",
      adapter: 2

    subconfig.vm.network "forwarded_port", guest: 15672, host: 15672

    subconfig.vm.hostname = "server.mb"
    subconfig.ssh.insert_key = SSH_INSERT_KEY

    subconfig.vm.provider "virtualbox" do |vb|
      vb.name = "MBServer"
      vb.memory = "1024"
      vb.cpus = 1
    end

    if Vagrant.has_plugin?("vagrant-proxyconf") && PROXY_ENABLED
      subconfig.proxy.http = PROXY_HTTP
      subconfig.proxy.https = PROXY_HTTPS
      subconfig.proxy.no_proxy = PROXY_EXCLUDE
    end
    
    $setup = <<-EOF
    pacman -Syu --noconfirm
    pacman -S erlang rabbitmq --noconfirm
    
    # /etc/rabbitmq/rabbitmq-env.conf
    # NODENAME=rabbit1@server
    # NO_IP_ADDRESS=0.0.0.0
    # NODE_PORT=5672
    # cluster_formation.peer_discovery_backend = classic_config
    # cluster_formation.classic_config.nodes.1 = rabbit1@server
    # cluster_formation.classic_config.nodes.2 = rabbit2@server
    
    systemctl enable rabbitmq
    systemctl start rabbitmq

    rabbitmqctl add_user #{MB_USER} #{MB_USER_PASS}
    rabbitmqctl add_vhost #{MB_VHOST_NAME}
    rabbitmqctl set_permissions -p #{MB_VHOST_NAME} #{MB_USER} #{MB_USER_HOST_PERM}
    rabbitmqctl set_user_tags #{MB_USER} administrator

    rabbitmq-plugins enable rabbitmq_management
    
    date > /etc/vagrant_provisioned_at
    EOF

    config.vm.provision "shell", inline: $setup
  end

  config.vm.define "web" do |subconfig|
    subconfig.vm.box = BOX_IMAGE

    subconfig.vm.network :private_network,
      ip: "#{WEB_SERVER_IP}",
      adapter: 2

    subconfig.vm.hostname = "server.web"
    subconfig.ssh.insert_key = SSH_INSERT_KEY

    subconfig.vm.provider "virtualbox" do |vb|
      vb.name = "WEBServer"
      vb.memory = "1024"
      vb.cpus = 1
    end

    if Vagrant.has_plugin?("vagrant-proxyconf") && PROXY_ENABLED
      subconfig.proxy.http = PROXY_HTTP
      subconfig.proxy.https = PROXY_HTTPS
      subconfig.proxy.no_proxy = PROXY_EXCLUDE
    end
    
    $setup = <<-EOF
    # TODO
    pacman -Syu --noconfirm
    
    date > /etc/vagrant_provisioned_at
    EOF

    config.vm.provision "shell", inline: $setup
  end
  
end
