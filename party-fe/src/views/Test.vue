<template>
<div class="tea-page">
  <el-form :model="form" :rules="rules">
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
  </div>

  <div v-if="!is_error" style="margin-top: 20px; background: #111; color: #0f0; padding: 4px 8px;min-height: 40px;">{{result}}</div>
  <div v-if="is_error" style="margin-top: 20px; background: #111; color: #f00; padding: 4px 8px;min-height: 40px;">{{result}}</div>
  
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
      
    }

    
    
  }

  
}
</script>