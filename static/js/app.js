import { createApp, ref, onMounted, computed } from 'vue';

const App = {
  setup() {
    const currentTab = ref('print'); // print, drivers, vehicles, works
    const statusMessage = ref('');
    
    const drivers = ref([]);
    const vehicles = ref([]);
    const works = ref([]);

    const loadData = async () => {
      try {
        const [dRes, vRes, wRes] = await Promise.all([
          fetch('/api/drivers'),
          fetch('/api/vehicles'),
          fetch('/api/works')
        ]);
        drivers.value = await dRes.json();
        vehicles.value = await vRes.json();
        works.value = await wRes.json();
      } catch (e) {
        statusMessage.value = 'Ошибка загрузки данных из БД!';
      }
    };

    onMounted(loadData);

    // --- Логика очереди печати ---
    const selectedDriver = ref('');
    const selectedVehicle = ref('');
    const selectedWork = ref('');
    const selectedDate = ref(new Date().toLocaleDateString('ru-RU'));
    
    const printQueue = ref([]);

    const addToQueue = () => {
      if (!selectedDriver.value || !selectedVehicle.value || !selectedWork.value) {
        alert('Пожалуйста, выберите все поля для путевого листа!');
        return;
      }
      printQueue.value.push({
        id: Date.now(),
        driver_id: parseInt(selectedDriver.value),
        vehicle_id: parseInt(selectedVehicle.value),
        work_type_id: parseInt(selectedWork.value),
        date: selectedDate.value,
        driver_name: drivers.value.find(d => d.id === selectedDriver.value)?.name,
        vehicle_name: vehicles.value.find(v => v.id === selectedVehicle.value)?.name,
        work_name: works.value.find(w => w.id === selectedWork.value)?.name,
      });
    };

    const removeFromQueue = (id) => {
      printQueue.value = printQueue.value.filter(q => q.id !== id);
    };

    const printBatch = async () => {
      if (printQueue.value.length === 0) {
        alert('Очередь пуста!');
        return;
      }
      statusMessage.value = 'Генерация путевых листов...';
      try {
        const response = await fetch('/api/print_batch', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ items: printQueue.value })
        });
        const data = await response.json();
        if (data.success && data.pdf_url) {
          statusMessage.value = 'Готово! Открываю PDF...';
          window.open(data.pdf_url + '?t=' + Date.now(), '_blank');
        } else {
          statusMessage.value = 'Ошибка: ' + data.message;
        }
      } catch (e) {
        statusMessage.value = 'Ошибка связи с сервером!';
      }
    };

    // --- Логика CRUD ---
    const formDriver = ref({ id: null, name: '', driving_license: '', tractor_license: '', snils: '' });
    const formVehicle = ref({ id: null, name: '', license_plate: '', sts: '', vehicle_type: 'Грузовой' });
    const formWork = ref({ id: null, name: '' });

    const editDriver = (d) => formDriver.value = { ...d, driving_license: d.driving_license || '', tractor_license: d.tractor_license || '', snils: d.snils || '' };
    const editVehicle = (v) => formVehicle.value = { ...v, license_plate: v.license_plate || '', sts: v.sts || '', vehicle_type: v.vehicle_type || 'Грузовой' };
    const editWork = (w) => formWork.value = { ...w };

    const clearForm = (type) => {
      if (type === 'driver') formDriver.value = { id: null, name: '', driving_license: '', tractor_license: '', snils: '' };
      if (type === 'vehicle') formVehicle.value = { id: null, name: '', license_plate: '', sts: '', vehicle_type: 'Грузовой' };
      if (type === 'work') formWork.value = { id: null, name: '' };
    };

    const saveItem = async (type, payload) => {
      const url = `/api/${type}s`;
      const method = payload.id ? 'PUT' : 'POST';
      try {
        await fetch(url, {
          method,
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload)
        });
        await loadData();
        clearForm(type);
      } catch (e) {
        alert('Ошибка сохранения!');
      }
    };

    const deleteItem = async (type, id) => {
      if (!confirm('Удалить запись?')) return;
      try {
        await fetch(`/api/${type}s`, {
          method: 'DELETE',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ id })
        });
        await loadData();
      } catch (e) {
        alert('Ошибка удаления!');
      }
    };

    return {
      currentTab, statusMessage,
      drivers, vehicles, works,
      selectedDriver, selectedVehicle, selectedWork, selectedDate,
      printQueue, addToQueue, removeFromQueue, printBatch,
      formDriver, formVehicle, formWork,
      editDriver, editVehicle, editWork, clearForm, saveItem, deleteItem
    };
  },
  template: `
    <div class="max-w-6xl mx-auto p-4 mt-8">
      
      <!-- Навигация -->
      <div class="flex space-x-2 mb-6 border-b border-gray-200 pb-2 overflow-x-auto">
        <button @click="currentTab = 'print'" :class="currentTab === 'print' ? 'text-brand border-b-2 border-brand font-semibold' : 'text-gray-500 hover:text-gray-700'" class="px-4 py-2 transition whitespace-nowrap">Печать листов</button>
        <button @click="currentTab = 'drivers'" :class="currentTab === 'drivers' ? 'text-brand border-b-2 border-brand font-semibold' : 'text-gray-500 hover:text-gray-700'" class="px-4 py-2 transition whitespace-nowrap">Водители</button>
        <button @click="currentTab = 'vehicles'" :class="currentTab === 'vehicles' ? 'text-brand border-b-2 border-brand font-semibold' : 'text-gray-500 hover:text-gray-700'" class="px-4 py-2 transition whitespace-nowrap">Техника</button>
        <button @click="currentTab = 'works'" :class="currentTab === 'works' ? 'text-brand border-b-2 border-brand font-semibold' : 'text-gray-500 hover:text-gray-700'" class="px-4 py-2 transition whitespace-nowrap">Виды работ</button>
      </div>

      <!-- Вкладка: Печать -->
      <div v-if="currentTab === 'print'" class="grid grid-cols-1 md:grid-cols-2 gap-6">
        
        <!-- Форма добавления в очередь -->
        <div class="bg-white p-6 rounded-xl shadow-md border border-gray-200">
          <h2 class="text-xl font-bold mb-4">Новый путевой лист</h2>
          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Дата</label>
              <input type="text" v-model="selectedDate" class="w-full px-4 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand" placeholder="ДД.ММ.ГГГГ">
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Водитель</label>
              <select v-model="selectedDriver" class="w-full px-4 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand">
                <option value="" disabled>Выберите водителя...</option>
                <option v-for="d in drivers" :key="d.id" :value="d.id">{{ d.name }}</option>
              </select>
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Техника</label>
              <select v-model="selectedVehicle" class="w-full px-4 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand">
                <option value="" disabled>Выберите технику...</option>
                <option v-for="v in vehicles" :key="v.id" :value="v.id">{{ v.name }} {{ v.license_plate ? '('+v.license_plate+')' : '' }}</option>
              </select>
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Вид работы</label>
              <select v-model="selectedWork" class="w-full px-4 py-2 border border-gray-300 rounded-lg outline-none focus:border-brand">
                <option value="" disabled>Выберите вид работы...</option>
                <option v-for="w in works" :key="w.id" :value="w.id">{{ w.name }}</option>
              </select>
            </div>
            <button @click="addToQueue" class="w-full bg-gray-100 hover:bg-gray-200 text-gray-800 font-semibold py-2 px-4 rounded-lg shadow-sm border border-gray-300 transition">
              + Добавить в список
            </button>
          </div>
        </div>

        <!-- Очередь -->
        <div class="bg-white p-6 rounded-xl shadow-md border border-gray-200 flex flex-col">
          <h2 class="text-xl font-bold mb-4">Список на печать ({{ printQueue.length }})</h2>
          <div class="flex-1 overflow-y-auto mb-4 border border-gray-200 rounded-lg bg-gray-50 p-2 min-h-[200px]">
            <div v-if="printQueue.length === 0" class="text-gray-400 text-center mt-10">Список пуст</div>
            <div v-for="item in printQueue" :key="item.id" class="bg-white p-3 rounded shadow-sm border border-gray-200 mb-2 flex justify-between items-center">
              <div class="text-sm">
                <div class="font-bold">{{ item.driver_name }}</div>
                <div class="text-gray-600">{{ item.vehicle_name }} | {{ item.work_name }}</div>
                <div class="text-xs text-gray-400">{{ item.date }}</div>
              </div>
              <button @click="removeFromQueue(item.id)" class="text-red-500 hover:text-red-700 p-2 text-xl">&times;</button>
            </div>
          </div>
          
          <button @click="printBatch" class="w-full bg-brand hover:bg-blue-600 text-white font-semibold py-3 px-4 rounded-lg shadow transition active:scale-[0.98]">
            🖨️ Сгенерировать PDF
          </button>
          <div v-if="statusMessage" class="text-center text-sm font-medium mt-2" :class="statusMessage.includes('Ошибка') ? 'text-red-500' : 'text-green-600'">
            {{ statusMessage }}
          </div>
        </div>
      </div>

      <!-- Вкладка: Водители -->
      <div v-if="currentTab === 'drivers'" class="bg-white p-6 rounded-xl shadow-md border border-gray-200">
        <h2 class="text-xl font-bold mb-4">Справочник водителей</h2>
        <form @submit.prevent="saveItem('driver', formDriver)" class="flex flex-wrap gap-2 mb-6 items-end">
          <div class="flex-1 min-w-[150px]"><label class="block text-xs text-gray-500">ФИО</label><input required v-model="formDriver.name" class="w-full border p-2 rounded"></div>
          <div class="flex-1 min-w-[150px]"><label class="block text-xs text-gray-500">В/У</label><input v-model="formDriver.driving_license" class="w-full border p-2 rounded"></div>
          <div class="flex-1 min-w-[150px]"><label class="block text-xs text-gray-500">Удост. тракториста</label><input v-model="formDriver.tractor_license" class="w-full border p-2 rounded"></div>
          <div class="flex-1 min-w-[150px]"><label class="block text-xs text-gray-500">СНИЛС</label><input v-model="formDriver.snils" class="w-full border p-2 rounded"></div>
          <button type="submit" class="bg-green-500 hover:bg-green-600 text-white px-4 py-2 rounded font-bold">{{ formDriver.id ? 'Сохранить' : 'Добавить' }}</button>
          <button type="button" v-if="formDriver.id" @click="clearForm('driver')" class="bg-gray-300 px-4 py-2 rounded">Отмена</button>
        </form>
        <div class="overflow-x-auto">
          <table class="w-full text-left border-collapse text-sm">
            <thead><tr class="bg-gray-100 border-b"><th class="p-2">ФИО</th><th class="p-2">В/У</th><th class="p-2">Тракторные</th><th class="p-2">СНИЛС</th><th class="p-2 w-24">Действия</th></tr></thead>
            <tbody>
              <tr v-for="d in drivers" :key="d.id" class="border-b hover:bg-gray-50">
                <td class="p-2">{{ d.name }}</td><td class="p-2">{{ d.driving_license }}</td><td class="p-2">{{ d.tractor_license }}</td><td class="p-2">{{ d.snils }}</td>
                <td class="p-2">
                  <button @click="editDriver(d)" class="text-blue-500 mr-2">✎</button>
                  <button @click="deleteItem('driver', d.id)" class="text-red-500">🗑</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Вкладка: Техника -->
      <div v-if="currentTab === 'vehicles'" class="bg-white p-6 rounded-xl shadow-md border border-gray-200">
        <h2 class="text-xl font-bold mb-4">Справочник техники</h2>
        <form @submit.prevent="saveItem('vehicle', formVehicle)" class="flex flex-wrap gap-2 mb-6 items-end">
          <div class="flex-1 min-w-[150px]">
            <label class="block text-xs text-gray-500">Тип</label>
            <select v-model="formVehicle.vehicle_type" class="w-full border p-2 rounded">
              <option value="Грузовой">Грузовой</option>
              <option value="Трактор">Трактор</option>
            </select>
          </div>
          <div class="flex-1 min-w-[150px]"><label class="block text-xs text-gray-500">Марка/Модель</label><input required v-model="formVehicle.name" class="w-full border p-2 rounded"></div>
          <div class="flex-1 min-w-[120px]"><label class="block text-xs text-gray-500">Госномер</label><input v-model="formVehicle.license_plate" class="w-full border p-2 rounded"></div>
          <div class="flex-1 min-w-[120px]"><label class="block text-xs text-gray-500">СТС / ПСМ</label><input v-model="formVehicle.sts" class="w-full border p-2 rounded"></div>
          <button type="submit" class="bg-green-500 hover:bg-green-600 text-white px-4 py-2 rounded font-bold">{{ formVehicle.id ? 'Сохранить' : 'Добавить' }}</button>
          <button type="button" v-if="formVehicle.id" @click="clearForm('vehicle')" class="bg-gray-300 px-4 py-2 rounded">Отмена</button>
        </form>
        <div class="overflow-x-auto">
          <table class="w-full text-left border-collapse text-sm">
            <thead><tr class="bg-gray-100 border-b"><th class="p-2">Тип</th><th class="p-2">Модель</th><th class="p-2">Госномер</th><th class="p-2">СТС / ПСМ</th><th class="p-2 w-24">Действия</th></tr></thead>
            <tbody>
              <tr v-for="v in vehicles" :key="v.id" class="border-b hover:bg-gray-50">
                <td class="p-2">{{ v.vehicle_type }}</td><td class="p-2">{{ v.name }}</td><td class="p-2">{{ v.license_plate }}</td><td class="p-2">{{ v.sts }}</td>
                <td class="p-2">
                  <button @click="editVehicle(v)" class="text-blue-500 mr-2">✎</button>
                  <button @click="deleteItem('vehicle', v.id)" class="text-red-500">🗑</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Вкладка: Виды работ -->
      <div v-if="currentTab === 'works'" class="bg-white p-6 rounded-xl shadow-md border border-gray-200">
        <h2 class="text-xl font-bold mb-4">Виды работ</h2>
        <form @submit.prevent="saveItem('work', formWork)" class="flex flex-wrap gap-2 mb-6 items-end">
          <div class="flex-1 min-w-[200px]"><label class="block text-xs text-gray-500">Наименование</label><input required v-model="formWork.name" class="w-full border p-2 rounded"></div>
          <button type="submit" class="bg-green-500 hover:bg-green-600 text-white px-4 py-2 rounded font-bold">{{ formWork.id ? 'Сохранить' : 'Добавить' }}</button>
          <button type="button" v-if="formWork.id" @click="clearForm('work')" class="bg-gray-300 px-4 py-2 rounded">Отмена</button>
        </form>
        <div class="overflow-x-auto">
          <table class="w-full text-left border-collapse text-sm">
            <thead><tr class="bg-gray-100 border-b"><th class="p-2">Вид работы</th><th class="p-2 w-24">Действия</th></tr></thead>
            <tbody>
              <tr v-for="w in works" :key="w.id" class="border-b hover:bg-gray-50">
                <td class="p-2">{{ w.name }}</td>
                <td class="p-2">
                  <button @click="editWork(w)" class="text-blue-500 mr-2">✎</button>
                  <button @click="deleteItem('work', w.id)" class="text-red-500">🗑</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

    </div>
  `
};

createApp(App).mount('#app');