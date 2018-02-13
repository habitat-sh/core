# -*- mode: ruby -*-
# vi: set ft=ruby :
Vagrant.configure("2") do |config|
  config.vm.box = "bento/ubuntu-17.10"

  config.vm.provider "virtualbox" do |vb|
    vb.memory = "4096"
    vb.cpus = "4"
  end

  config.vm.provider "vmware_fusion" do |v|
    v.vmx["memsize"] = "4096"
    v.vmx["numvcpus"] = "4"
  end

  config.vm.provision "shell", inline: <<-SCRIPT
curl -o /tmp/install.sh https://raw.githubusercontent.com/habitat-sh/habitat/master/components/hab/install.sh
SCRIPT
  config.vm.provision "shell", path: "https://raw.githubusercontent.com/habitat-sh/habitat/master/support/linux/install_dev_0_ubuntu_latest.sh"
  config.vm.provision "shell", path: "https://raw.githubusercontent.com/habitat-sh/habitat/master/support/linux/install_dev_9_linux.sh"
end
