<template>
  <el-dialog
    title="Login"
    :visible="visible"
    width="70%"
    :close-on-click-modal="false"
    custom-class="tea-modal"
    :destroy-on-close="true"
    @opened="openHandler()"
    @close="close()"
  >

    <i v-if="!param || loading" class="el-icon-loading" style="display: block; width: 40px; height: 40px;font-size: 40px; margin: 0 auto;"></i>

    <div v-if="!loading" style="text-align:left;">
      <div style="font-size: 15px;" v-if="layer1_account.address">

        <h4>Please confirm the allowed permissions.</h4>

        <div>
          <el-checkbox v-model="read" disabled>Read</el-checkbox>
          <el-checkbox v-model="move">Move</el-checkbox>
          <el-checkbox v-model="consume">Consume</el-checkbox>
          <el-checkbox v-model="withdraw">Withdraw</el-checkbox>
        </div>
      </div>
      <!-- <el-button v-if="layer1_account.address" type="primary" @click="confirm()">Login</el-button> -->

      <p style="font-size: 16px; color: #f00;" v-if="!layer1_account.address">
        Please select account from Polkadot extention.
      </p>
    </div>
    

    <span slot="footer" class="dialog-footer">
      <el-button size="small" @click="close()">Cancel</el-button>
      <el-button size="small" type="primary" @click="confirm()">Login</el-button>
    </span>

  </el-dialog>


</template>
<script>
import { mapState, mapGetters } from 'vuex';
import {stringToHex, stringToU8a, hexToU8a, hexToString} from 'tearust_layer1';
import store from '../../store/index';
import utils from '../../tea/utils';
import Base from '../../workflow/Base';
import {_} from 'tearust_utils';
import bbs from '../bbs';
import user from '../user';
export default {
  data(){
    return {
      loading: true,
      form: {
        
      },
      read: true,
      move: true,
      consume: true,
      withdraw: true,
    };
  },
  computed: {
    ...mapGetters([
      'layer1_account'
    ]),
    ...mapState('modal', {
      visible:state => store.state.modal.login.visible,
      param: state => store.state.modal.login.param,
    })
  },

  methods: {
    reset(){
      this.loading = true;
      this.form = {
        
      };
    },
    close(){
      this.$store.commit('modal/close', 'login');
      _.delay(()=>{
        this.reset();
      }, 500);
    },
    async confirm(){
      const cb = utils.mem.get('login');

      const tmp = [];
      if(this.read) tmp.push('read');
      if(this.move) tmp.push('move');
      if(this.consume) tmp.push('consume');
      if(this.withdraw) tmp.push('withdraw');
      if(cb){
        await cb(tmp.join("_"), this.close);
      }
    },
    async openHandler(){
      this.wf = new Base();
      await this.wf.init();
      const layer1_instance = this.wf.getLayer1Instance();
      let api = layer1_instance.getApi();

      
      // if(this.layer1_account.address){
      //   await user.loginPrepare(layer1_instance, this.layer1_account.address);
      // }

      this.loading = false;
    }
  }
}
</script>