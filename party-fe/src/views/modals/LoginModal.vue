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
      <p style="font-size: 15px;" v-if="layer1_account.address">

        Click the button below to login with {{layer1_account.address}}.
      </p>
      <el-button v-if="layer1_account.address" type="primary" @click="confirm()">Login</el-button>

      <p style="font-size: 16px; color: #f00;" v-if="!layer1_account.address">
        Please select account from Polkadot extention.
      </p>
    </div>
    

    <span slot="footer" class="dialog-footer">
      <el-button size="small" @click="close()">Cancel</el-button>
      <!-- <el-button size="small" type="primary" @click="confirm()">Login</el-button> -->
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
        
      }
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
      await user.login(this.layer1_account.address);
      const cb = utils.mem.get('login');
      if(cb){
        await cb(this.close);
      }
    },
    async openHandler(){
      this.wf = new Base();
      await this.wf.init();
      const layer1_instance = this.wf.getLayer1Instance();
      let api = layer1_instance.getApi();

      
      if(this.layer1_account.address){
        await user.loginPrepare(layer1_instance, this.layer1_account.address);
      }

      this.loading = false;
    }
  }
}
</script>