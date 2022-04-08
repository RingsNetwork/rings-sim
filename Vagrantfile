Vagrant.configure("2") do |config|
  config.vm.provider "parallels"
  config.vm.box = "mpasternak/focal64-arm"
  config.vm.synced_folder "etc", "/host/etc"
  config.vm.synced_folder "sim", "/home/vagrant/sim"
  config.vm.provision "bootstrap", type: "shell" do |s|
    s.path = "setup.sh"
  end
end
