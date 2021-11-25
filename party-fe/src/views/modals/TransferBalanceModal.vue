<template>
  <el-dialog
    :title="'Send' | addTea"
    :visible="visible"
    width="70%"
    :close-on-click-modal="false"
    custom-class="tea-modal"
    :destroy-on-close="true"
    @close="close()"
  >

    <el-form :model="form" label-width="150px">
      <el-form-item label="Receiver's address">
        <el-input v-model="form.address"></el-input>
      </el-form-item>
      <el-form-item label="TEA amount to send">
        <el-input v-model="form.amount" ></el-input>
      </el-form-item>
    </el-form>

    <span slot="footer" class="dialog-footer">
      <el-button size="small" @click="close()">Cancel</el-button>
      <el-button size="small" type="primary" @click="confrim()">Send</el-button>
    </span>

  </el-dialog>


</template>
<script>
import { mapState } from 'vuex';
import store from '../../store/index';
import utils from '../../tea/utils';
export default {
  data(){
    return {
      form: {
        address: null,
        amount: null,
      }
    };
  },
  computed: {
    ...mapState('modal', {
      visible:state => store.state.modal.transfer_balance.visible,
      param: state => store.state.modal.transfer_balance.param,
    })
  },

  methods: {
    reset(){
      this.form = {
        address: null,
        amount: null,
      };
    },
    close(){
      this.reset();
      this.$store.commit('modal/close', 'transfer_balance');
    },
    async confrim(){
      const cb = utils.mem.get('transfer_balance');
      if(cb){
        const form = {
          address: this.form.address,
          amount: parseFloat(this.form.amount),
        };
        await cb(form, ()=>{
          this.close();
        });
      }
    }
  }
}
</script>