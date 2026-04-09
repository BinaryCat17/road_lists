import { createApp, ref, onMounted } from 'vue';

const App = {
  setup() {
    const selectedDriver = ref('');
    const selectedVehicle = ref('');
    const selectedWork = ref('');
    const statusMessage = ref('');
    
    const drivers = ref([]);
    const vehicles = ref([]);
    const works = ref([]);

    // Загрузка данных из БД при старте
    onMounted(async () => {
      try {
        const dRes = await fetch('/api/drivers');
        drivers.value = await dRes.json();
        
        const vRes = await fetch('/api/vehicles');
        vehicles.value = await vRes.json();
        
        const wRes = await fetch('/api/works');
        works.value = await wRes.json();
      } catch (e) {
        statusMessage.value = 'Ошибка загрузки данных из БД!';
      }
    });

    const printWaybill = async () => {
      if (!selectedDriver.value || !selectedVehicle.value || !selectedWork.value) {
        statusMessage.value = 'Пожалуйста, заполните все поля!';
        return;
      }
      
      statusMessage.value = 'Генерация путевого листа...';

      try {
        const response = await fetch('/api/print', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            driver_id: parseInt(selectedDriver.value),
            vehicle_id: parseInt(selectedVehicle.value),
            work_type_id: parseInt(selectedWork.value)
          })
        });

        const data = await response.json();
        if (data.success && data.pdf_url) {
          statusMessage.value = 'Готово! Открываю PDF...';
          // Добавляем timestamp чтобы браузер не кешировал PDF
          window.open(data.pdf_url + '?t=' + Date.now(), '_blank');
        } else {
          statusMessage.value = 'Ошибка: ' + data.message;
        }
      } catch (e) {
        statusMessage.value = 'Ошибка связи с сервером!';
      }
    };

    return {
      selectedDriver, selectedVehicle, selectedWork,
      drivers, vehicles, works,
      printWaybill, statusMessage
    };
  },
  template: `
    <div class="max-w-2xl mx-auto p-6 mt-10 bg-white rounded-xl shadow-md border border-gray-200">
      <h1 class="text-2xl font-bold text-gray-800 mb-6 text-center">Генератор Путевых Листов</h1>
      
      <div class="space-y-5">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Водитель</label>
          <select v-model="selectedDriver" class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-brand outline-none transition">
            <option value="" disabled>Выберите водителя...</option>
            <option v-for="d in drivers" :key="d.id" :value="d.id">{{ d.name }}</option>
          </select>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Техника</label>
          <select v-model="selectedVehicle" class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-brand outline-none transition">
            <option value="" disabled>Выберите технику...</option>
            <option v-for="v in vehicles" :key="v.id" :value="v.id">{{ v.name }}</option>
          </select>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Вид работы</label>
          <select v-model="selectedWork" class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-brand outline-none transition">
            <option value="" disabled>Выберите вид работы...</option>
            <option v-for="w in works" :key="w.id" :value="w.id">{{ w.name }}</option>
          </select>
        </div>

        <div class="pt-4">
          <button @click="printWaybill" class="w-full bg-brand hover:bg-blue-600 text-white font-semibold py-3 px-4 rounded-lg shadow transition active:scale-[0.98]">
            🖨️ Сформировать и Печатать
          </button>
        </div>
        
        <div v-if="statusMessage" class="text-center text-sm font-medium" :class="statusMessage.includes('Ошибка') ? 'text-red-500' : 'text-green-600'">
          {{ statusMessage }}
        </div>
      </div>
    </div>
  `
};

createApp(App).mount('#app');
