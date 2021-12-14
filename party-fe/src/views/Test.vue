<template>
<div class="tea-page">
  <el-form :model="form">
    <el-form-item label="Action">
      <el-input v-model="form.action"></el-input>
    </el-form-item>
    <el-form-item label="Payload">
      <el-input type="textarea" :rows="8" v-model="form.payload"></el-input>
    </el-form-item>
    <el-form-item label="UUID">
      <el-input v-model="form.uuid"></el-input>
    </el-form-item>

  </el-form>

  <div style="text-align:right;">
    <el-button type="primary" plain @click="generate_uuid()">Generate UUID</el-button>

    <el-button type="primary">Request action</el-button>
    <el-button type="primary">Query result</el-button>
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
import request from '../request';
import bbs from './bbs';

export default {

  data(){
    return {
      form: {
        action: 'test_action',
        payload: '',
        uuid: '',
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
    }

    
    
  }

  
}
</script>