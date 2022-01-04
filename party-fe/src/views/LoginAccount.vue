<template>
<div class="tea-page">

  <div class="tea-card">
    <i class="x-icon ">
      <img src="/fav.png" />
    </i>
    

    <div class="x-list" style="width:100%;">
      <div class="x-item">
        <b>{{'Name' | cardTitle}}</b>
        <span>{{layer1_account ? layer1_account.name : ''}}</span>
      </div>
      <div class="x-item">
        <b>{{'Address' | cardTitle}}</b>
        <span>
          <font class="js_need_copy">{{layer1_account ? layer1_account.address : ''}}</font>
          <!-- <span title="copy" data-clipboard-target=".js_need_copy" style="margin-left: 5px;" class="iconfont tea-icon-btn icon-copy js_copy"></span> -->
          <!-- <span @click="showAddressQrcode(layer1_account.address)" style="margin-left: 5px;" title="qrcode" class="iconfont tea-icon-btn icon-qr_code"></span> -->
          
        </span>

      </div>
      <div class="x-item">
        <b>{{'My TEA' | cardTitle}}</b>
        <span :inner-html.prop="layer1_account ? layer1_account.balance : '' | teaIcon"></span>
      </div>

      <div class="x-item">
        <b>{{'My tapp balance'}}</b>
        <span :inner-html.prop="tapp_balance===null ? '...' : tapp_balance"></span>
      </div>
      

     

      <div class="x-bottom">


        <el-button v-if="layer1_account" @click="rechargeHandler()">Recharge with TEA</el-button>


      </div>

    </div>

    <!-- <div class="x-right">
      
    </div> -->

  </div>
  



  

</div>
</template>
<script>
import Vue from 'vue';
import SettingAccount from '../workflow/SettingAccount';
import {_} from 'tearust_utils';
import {helper, numberToHex} from 'tearust_layer1';
import utils from '../tea/utils';
import { mapGetters, mapState } from 'vuex';

import PubSub from 'pubsub-js';
import ClipboardJS from 'clipboard';
import TeaIconButton from '../components/TeaIconButton';
import request from '../request';
import bbs from './bbs';

export default {
  components: {

    TeaIconButton,
  },
  data(){
    return {
      tapp_balance: null,
    };
  },

  computed: {
    ...mapGetters([
      'layer1_account'
    ]),
  },

  async created(){
    this.initCopyEvent();
  },
  beforeDestroy(){
    this.clipboard && this.clipboard.destroy();
  },
  
  async mounted(){
    this.$root.loading(true);

    this.wf = new SettingAccount();
    await this.wf.init();
    await this.refreshAccount();

    this.$root.loading(false);
    

    // const layer1_instance = this.wf.getLayer1Instance();
    // const api = layer1_instance.getApi();
    
  },

  methods: {
    showSelectLayer1(){
      this.wf.showSelectLayer1Modal();
    },


    async rechargeHandler(){
      bbs.topupFromLayer1(this, async ()=>{
        await this.refreshAccount(true);
      });
    },

    async refreshAccount(flag=false){
      flag && this.$root.loading(true);
      await this.wf.refreshCurrentAccount();

      const layer1_account = this.layer1_account;

      await this.queryTokenBalance();
      
      
      flag && this.$root.loading(false);
    },

    async queryTokenBalance(){
      try{
        this.tapp_balance = await bbs.query_balance({
          address: this.layer1_account.address,
        });
      }catch(e){
        console.error(e);
      }
      
    },


    async transferBalance(){
      const layer1_instance = this.wf.getLayer1Instance();
      const api = layer1_instance.getApi();

      this.$store.commit('modal/open', {
        key: 'transfer_balance',
        param: {},
        cb: async (form, closeFn)=>{
          this.$root.loading(true);
          try{
            const {address, amount} = form;

            await this.wf.transferBalance(address, amount);

            closeFn();
            await this.refreshAccount();
          }catch(e){
            this.$root.showError(e);
          }
          this.$root.loading(false);
        },
      });
    },

    

    clickRefreshBtn(){
      utils.publish('refresh-current-account__account');
      utils.publish('refresh-current-account__'+this.tab);
    },

    

    showAddressQrcode(address){
      PubSub.publish('tea-qrcode-modal', {
        info: null,
        visible: true,
        text: address,
      });
    },

    initCopyEvent(){
      const clipboard = new ClipboardJS('.js_copy');
      this.clipboard = clipboard;
      clipboard.on('success', (e)=>{
        e.clearSelection();
        this.$root.success('Copied');
      });

      clipboard.on('error', (e)=>{
      });
    },

    
    
  }

  
}
</script>

<style lang="scss">
.tea-page{
  .t-major-financial{
    margin-top: 5px;
    text-align: right;
    padding-right: 8px;

    b{
      color: #35a696;
    }
    span{
      margin: 0 5px;
      color: #c9c9c9;
    }
    span.iconfont{
      color: #35a696;
      margin: 0;
    }
  }
}

</style>