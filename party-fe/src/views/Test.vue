<template>
<div class="tea-page">
  <!-- <el-form :model="form" :rules="rules">
    <el-form-item label="Action" prop="action">
      <el-input v-model="form.action"></el-input>
    </el-form-item>
    <el-form-item label="Payload" prop="payload">
      <el-input type="textarea" :rows="4" v-model="form.payload"></el-input>
    </el-form-item>
    <el-form-item label="UUID" prop="uuid">
      <el-input v-model="form.uuid"></el-input>
    </el-form-item>

  </el-form>

  <div style="text-align:right;">
    <el-button type="primary" plain @click="generate_uuid()">Generate UUID</el-button>

    <el-button type="primary" @click="test_action()">Request action</el-button>
    <el-button type="primary" @click="test_result()">Query result</el-button>
  </div> -->

  <div style="text-align:left;">
    <el-button v-if="!user || !user.isLogin" type="primary" @click="login_action()">Login</el-button>
    <el-button v-if="user && user.isLogin" type="primary" @click="logout_action()">Logout</el-button>
    <el-divider />
    <el-button type="primary" @click="topup_action()">Topup</el-button>
    <el-button type="danger" @click="query_balance_action()">Query balance</el-button>
    <el-button type="primary" @click="withdraw_action()">Withdraw</el-button>
    <el-divider />
    <el-button type="primary" @click="update_profile_action()">Update TApp profile</el-button>
    <el-divider />
  
  </div>

  <div v-if="!is_error" v-html="result" style="margin-top: 20px; background: #111; color: #0f0; padding: 4px 8px;min-height: 40px; line-height:20px; word-break:break-all;"></div>
  <div v-if="is_error" v-html="result" style="margin-top: 20px; background: #111; color: #f00; padding: 4px 8px;min-height: 40px; line-height:20px; word-break:break-all;">{{result}}</div>
  
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
import request from '../request';
import bbs from './bbs';
import user from './user';

export default {

  data(){
    return {
      form: {
        action: 'test_action',
        payload: '',
        uuid: '',
      },
      result: '',
      is_error: false,
      rules: {
        action: [{required: true}],
        payload: [{required: true}],
        uuid: [{required: true}],
      }
    };
  },

  computed: {
    ...mapState([
      'user', 'bbs',
    ]),
    ...mapGetters([
      'layer1_account'
    ]),
  },


  async mounted(){
    this.$root.loading(true);

    this.wf = new SettingAccount();
    await this.wf.init();

    this.$root.loading(false);
    

    // const layer1_instance = this.wf.getLayer1Instance();
    // const api = layer1_instance.getApi();
    
  },

  methods: {
    generate_uuid(){
      this.form.uuid = bbs.test.get_uuid();
    },
    async test_action(){
      const {payload, uuid, action} = this.form;

      const json = utils.parseJSON(payload);
      console.log(111, json);
      if(!json){
        alert('Invalid json payload');
        return;
      }

      if(!uuid){
        alert('Invalid UUID');
        return;
      }

      if(!action){
        alert("Invalid action");
        return;
      }

      const rs = await bbs.test.request(uuid, json, action);
      this.result = JSON.stringify(rs);

    },
    show_result(msg, is_error=false){
      this.result = msg;
      this.is_error = is_error;
    },
    async test_result(){
      const {uuid} = this.form;
      if(!uuid){
        alert('Invalid UUID');
        return;
      }
      try{
        const rs = await bbs.test.result(uuid);
        this.show_result(JSON.stringify(rs));
      }catch(e){
        this.show_result(e, true);
      }
      
    },

    setLog(init_msg){
      this.show_result(init_msg);
      bbs.setLog((msg)=>{
        this.show_result(msg+'<br/>'+this.result);
      })
    },
    

    async login_action(){
      this.setLog('start login action...');
      await user.showLoginModal(this);

    },
    async logout_action(){
      this.setLog('start logout action...');
      await user.logout(this.layer1_account.address);
    },
    async topup_action(){
      this.setLog('start topup action...');
      await bbs.topupFromLayer1(this, async ()=>{
        bbs.log('layer1 topup success.')
      });
    },
    async query_balance_action(){
      this.setLog("query balance action...");
      try{
        const balance = await bbs.query_balance({
          address: this.layer1_account.address,
        });
      }catch(e){
        bbs.log(e);
      }
    },
    async update_profile_action(){
      this.setLog("start update tapp profile action...");
      try{
        const rs = await bbs.updateTappProfile(this.layer1_account.address);
      }catch(e){
        bbs.log(e);
      }
      
    },
    async withdraw_action(){
      
    }


    
    
  }

  
}
</script>