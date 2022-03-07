<template>
<div class="c-pageheader">

<el-menu active-text-color="#35a696" :default-active="activeIndex" class="p-header" @select="handleSelect" mode="horizontal">
  <a href="javascript:void(0)" onClick="location.reload()" style="float:left;">
    <el-image
      src="https://wallet.teaproject.org/tea_logo/logo.png"
      fit="fit">
    </el-image>
    
    <!-- <b class="lg">{{chain.current_block_hash ? chain.current_block : ''}}</b> -->
  </a>
  


  <div style="margin-left: 50px;" class="el-menu-item">
    <el-dropdown trigger="click" @command="handleCommand">
      <el-button size="small" type="primary" round style="font-size: 14px; " @click="clickSelectAccount()">
        {{layer1_account.name || 'Select account'}}
        <!-- <i class="el-icon-arrow-down el-icon--right"></i> -->
      </el-button>
      <el-dropdown-menu slot="dropdown">
        <el-dropdown-item v-for="(item, i) of all_account" :key="i" :command="item">
          <span v-if="layer1_account && layer1_account.address!==item.address">{{item.ori_name}}</span>
        </el-dropdown-item>
        
      </el-dropdown-menu>
    </el-dropdown>

    <el-button style="margin-left: 10px;" @click="loginOrLogout()" type="text">{{user ? 'Logout' : 'Login'}}</el-button>
  </div>
  

  <!-- <el-menu-item index="/test">{{'TEST'}}</el-menu-item> -->

  
  
  <el-menu-item index="/log">{{'Log'}}</el-menu-item>

  <el-menu-item v-if="user && user.isLogin" index="/my_notification">{{'Inbox'}}</el-menu-item>

  <!-- <el-menu-item index="/tapp_profile">{{'Profile'}}</el-menu-item> -->
  <el-menu-item v-if="user && user.isLogin" index="/profile">{{'My assets'}}</el-menu-item>


  <!-- <el-menu-item style="margin-right: 30px;" index="/home">{{bbs.channel || '...'}}</el-menu-item> -->
  <el-menu-item style="margin-right: 30px;" index="/home">Public Board</el-menu-item>
  <el-menu-item index="/channel">{{'Free Board'}}</el-menu-item>

  
</el-menu>

<div class="t-state" :class="'x_'+connected"></div>

<div v-if="top_log" style="height: 36px; width: 1080px; margin: 0 auto;">
  <el-alert
    effect="dark"
    @close="top_log=null"
    center
    :closable="true"
    :title="top_log"
    :type="top_log_level">
  </el-alert>
</div>

</div>
  

</template>
<script>
import {mapGetters, mapState} from 'vuex';
import Base from '../workflow/Base';
import _ from 'lodash';
import utils from '../tea/utils';
import user from '../views/user';
import bbs from '../views/bbs';
export default {
  data() {
    return {
      activeIndex: null,
      connected: 0,
      has_seed_pool: false,

      all_account: [],
      no_plugin_account: false,

      top_log: null,
      top_log_level: 'error',
    };
  },
  watch: {
    '$route': {
      immediate: true,
      handler (to, from){
        let name = to.path;

        this.activeIndex = name;

        console.log(111, to, this.layer1_account.address, this.user)
        if(to.meta && to.meta.needLogin && this.layer1_account.address && (!this.user || !this.user.isLogin)){
          this.$router.push('/home');
        }
      }
    },

  },
  computed: {
    ...mapState([
      'chain', 'user', 'bbs',
    ]),
    ...mapGetters(['layer1_account']),
    // ...mapState([
    //   'chain'
    // ]),
  },
  methods: {
    handleSelect(key, keyPath) {
      if(key === 'lang'){
        this.changeLang();
        return false;
      }
      
      if(this.$route.path !== key){
        this.$router.push(key);
      }
      
    },
    changeLang(){
      if(this.$i18n.locale === 'en'){
        window.changeLanguage('zh');
      }
      else{
        window.changeLanguage('en');
      }

    },

    async initAllPluginAccount(wf){
      const layer1_instance = wf.getLayer1Instance();
      let tmp = await wf.getAllLayer1Account();
      tmp = _.map(tmp||[], (item)=>{
        (async ()=>{
          // item.balance = await layer1_instance.getAccountBalance(item.address);
          item.ori_name = _.clone(item.name);
          item.name = item.name + '  -  ' + item.balance;
        })();
        return item;
      });

      if(tmp.length < 1){
        this.no_plugin_account = true;
      }
      else{
        this.no_plugin_account = false;
      }

      this.all_account = tmp;
    },

    async handleCommand(item){

      this.$store.commit('set_account', item);
      location.reload(true);
      
      // if(this.wf){
      //   this.$root.loading(true);
      //   bbs.top_log(null);
      //   await this.wf.refreshCurrentAccount();
        
      //   this.$root.loading(false);
      // }

    },
    async clickSelectAccount(){
      if(this.no_plugin_account){
        const html = `
          <p style="font-size: 15px;">Please add an account or <a target="_blank" href="https://teaproject.org/#/doc_list/%2FFAQ%2Fhow_to_install_polkadot_extension.md ">install polkadot browser extension</a></p>
        `;
        this.$alert(html, {
          dangerouslyUseHTMLString: true,
        });
      }
    },

    async loginOrLogout(){
      if(!this.user){
        await user.showLoginModal(this);
      }
      else{
        await user.logout(this.layer1_account.address);
        this.$root.success('Logout success.');
        _.delay(()=>{
          location.reload(true);
        }, 2000);
        
      }
    }
    
  },
  async mounted(){
    bbs.set_global_log(this);

    const id = bbs.getTappId();
    const channel = decodeURIComponent(utils.urlParam('v'));
    
    console.log('tapp id => '+id);
    console.log('main channel => '+channel);

    this.$store.commit('set_bbs', {
      id,
      channel,
    });


    await (new Base()).init()
    let time = 500;

    const loop = async (cb)=>{

      try{
        const wf = new Base();
        const connected = wf.layer1.isConnected();
        if(connected !== this.connected){
          this.connected = connected;

          if(this.connected === 2){
            this.wf = wf;
            this.initAllPluginAccount(wf);
            cb();
          }
          
        }
        
        if(this.connected > 0){
          time = 2000;
        }

      }catch(e){
        this.connected = 0;
      }
     
      _.delay(()=>{
        loop(cb);
      }, time);
    };

    loop(async ()=>{

      await this.$store.dispatch('init_user');

      // if(this.wf && this.chain && this.chain.metadata){
      //   await this.wf.init();
      //   const block = await this.wf.getCurrentBlock();
      // }

      const tapp = await bbs.getTappDetail(this);
      console.log("tapp detail =>", tapp);
      this.$store.commit('set_bbs', {
        id,
        channel,
        tapp,
      });
    });

    utils.register('top_log', (key, param)=>{
      this.top_log = param.top_log;
      this.top_log_level = param.top_log_level;
    });


  }
}
</script>
<style lang="scss">
.c-pageheader{
  position: sticky;
  top: 0;
  display: block;
  background: #fff;
  z-index: 99;
  text-align: center;
}

.p-header{
  padding: 0 0 10px;
  width: 1080px;
  margin: 0 auto !important;
  .lg{
    font-size: 20px;
    color: #333;
    position: relative;
    vertical-align: top;
    top: 20px;
    left: 90px;
  }
  .el-image{
    width: 60px; 
    height: 60px;
    width: 90px;
    height: 90px;
    position: absolute;
    top: -10px;
  }
  
}
.el-menu--horizontal > .el-menu-item{
  float: right !important;
  padding: 10px !important;
  font-size: 17px !important;
}
.el-menu--horizontal > .el-submenu{
  float: right !important;
  
}
.el-menu.el-menu--horizontal{
  border-bottom: none;
}

.t-state{
  height: 2px;
  width: 100%;
  display: block;
  
  &.x_0{
    background: red;
  }
  &.x_1{
    background: yellow;
  }
  &.x_2{
    background: #35a696;
  }

}
</style>