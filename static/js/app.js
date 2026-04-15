import { createApp, ref, onMounted, computed } from 'vue';

const App = {
  setup() {
    // Auth state
    const loading = ref(true);
    const isAuthenticated = ref(false);
    const user = ref(null);

    // App state
    const currentTab = ref('print');
    const statusMessage = ref('');
    
    const drivers = ref([]);
    const vehicles = ref([]);
    const companySettings = ref({ 
      company_name: '', 
      company_address: '', 
      company_inn: '', 
      dispatcher_name: '', 
      mechanic_name: '', 
      medic_name: '' 
    });

    const defaultValues = ref({
      customer: '', 
      loading_point: '', 
      unloading_point: '', 
      cargo: '', 
      trips: '', 
      distance: '', 
      tons: '', 
      arrival_time: '',
      field_object: '', 
      field_area: '', 
      field_norm: '', 
      field_fact: '', 
      field_motohours: ''
    });

    // Check auth on mount
    const checkAuth = async () => {
      try {
        const res = await fetch('/api/me', { credentials: 'include' });
        if (res.ok) {
          user.value = await res.json();
          isAuthenticated.value = true;
          await loadData();
        } else {
          isAuthenticated.value = false;
          // Redirect to login if not authenticated
          if (window.location.pathname === '/' || window.location.pathname === '/index.html') {
            window.location.href = '/login.html';
            return;
          }
        }
      } catch (e) {
        console.error('Auth check failed:', e);
        isAuthenticated.value = false;
      } finally {
        loading.value = false;
      }
    };

    const userInitials = computed(() => {
      if (!user.value?.name) return '?';
      return user.value.name.split(' ').map(n => n[0]).join('').toUpperCase().slice(0, 2);
    });

    const loadData = async () => {
      try {
        const [dRes, vRes, sRes, defRes] = await Promise.all([
          fetch('/api/drivers', { credentials: 'include' }),
          fetch('/api/vehicles', { credentials: 'include' }),
          fetch('/api/settings', { credentials: 'include' }),
          fetch('/api/defaults', { credentials: 'include' })
        ]);
        
        if (!dRes.ok || !vRes.ok || !sRes.ok || !defRes.ok) {
          if (dRes.status === 401 || vRes.status === 401) {
            isAuthenticated.value = false;
            window.location.href = '/login.html';
            return;
          }
          throw new Error('Failed to load data');
        }

        drivers.value = await dRes.json();
        vehicles.value = await vRes.json();
        const s = await sRes.json();
        if (s) companySettings.value = { ...companySettings.value, ...s };
        const d = await defRes.json();
        if (d) {
          defaultValues.value = { ...defaultValues.value, ...d };
          applyDefaults();
        }
      } catch (e) {
        console.error('Load data error:', e);
        statusMessage.value = 'Ошибка загрузки данных из БД!';
      }
    };

    onMounted(checkAuth);

    // --- Print queue logic ---
    const selectedDriver = ref('');
    const selectedVehicle = ref('');
    const selectedDate = ref(new Date().toLocaleDateString('ru-RU'));
    const tractorMode = ref('cargo');

    const selectedVehicleObj = computed(() => vehicles.value.find(v => v.id == selectedVehicle.value));
    const isTractorSelected = computed(() => selectedVehicleObj.value?.vehicle_type === 'Трактор');

    const makeTaskFromDefaults = () => ({
      customer: defaultValues.value.customer || '',
      loading_point: defaultValues.value.loading_point || '',
      unloading_point: defaultValues.value.unloading_point || '',
      cargo: defaultValues.value.cargo || '',
      trips: defaultValues.value.trips || '',
      distance: defaultValues.value.distance || '',
      tons: defaultValues.value.tons || '',
      arrival_time: defaultValues.value.arrival_time || ''
    });

    const taskRows = ref([makeTaskFromDefaults()]);
    const addTaskRow = () => {
      if (taskRows.value.length < 3) taskRows.value.push(makeTaskFromDefaults());
    };
    const removeTaskRow = (idx) => {
      if (taskRows.value.length > 1) taskRows.value.splice(idx, 1);
    };

    const makeFieldRowFromDefaults = () => ({
      object: defaultValues.value.field_object || '',
      area: defaultValues.value.field_area || '',
      norm: defaultValues.value.field_norm || '',
      fact: defaultValues.value.field_fact || '',
      motohours: defaultValues.value.field_motohours || ''
    });

    const fieldRows = ref([makeFieldRowFromDefaults()]);
    const addFieldRow = () => {
      if (fieldRows.value.length < 3) fieldRows.value.push(makeFieldRowFromDefaults());
    };
    const removeFieldRow = (idx) => {
      if (fieldRows.value.length > 1) fieldRows.value.splice(idx, 1);
    };

    const applyDefaults = () => {
      taskRows.value = [makeTaskFromDefaults()];
      fieldRows.value = [makeFieldRowFromDefaults()];
    };

    const printQueue = ref([]);

    const addToQueue = () => {
      if (!selectedDriver.value || !selectedVehicle.value) {
        alert('Пожалуйста, выберите водителя и технику!');
        return;
      }
      let tasks = [];
      let mode = null;
      if (isTractorSelected.value && tractorMode.value === 'field') {
        mode = 'field';
        tasks = fieldRows.value.map(r => ({
          customer: '',
          loading_point: r.object,
          unloading_point: '',
          cargo: '',
          trips: r.fact,
          distance: r.area,
          tons: r.norm,
          arrival_time: r.motohours,
        }));
      } else {
        tasks = taskRows.value.map(t => ({ ...t }));
      }
      printQueue.value.push({
        id: Date.now(),
        driver_id: parseInt(selectedDriver.value),
        vehicle_id: parseInt(selectedVehicle.value),
        date: selectedDate.value,
        driver_name: drivers.value.find(d => d.id == selectedDriver.value)?.name,
        vehicle_name: selectedVehicleObj.value?.name,
        tasks: tasks,
        tractor_mode: mode,
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
          credentials: 'include',
          body: JSON.stringify({ items: printQueue.value })
        });
        
        if (response.status === 401) {
          isAuthenticated.value = false;
          window.location.href = '/login.html';
          return;
        }
        
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

    // --- CRUD ---
    const formDriver = ref({ id: null, name: '', driving_license: '', tractor_license: '', snils: '' });
    const formVehicle = ref({ id: null, name: '', license_plate: '', sts: '', vehicle_type: 'Грузовой', category: '' });

    const editDriver = (d) => formDriver.value = { ...d, driving_license: d.driving_license || '', tractor_license: d.tractor_license || '', snils: d.snils || '' };
    const editVehicle = (v) => formVehicle.value = { ...v, license_plate: v.license_plate || '', sts: v.sts || '', vehicle_type: v.vehicle_type || 'Грузовой', category: v.category || '' };

    const clearForm = (type) => {
      if (type === 'driver') formDriver.value = { id: null, name: '', driving_license: '', tractor_license: '', snils: '' };
      if (type === 'vehicle') formVehicle.value = { id: null, name: '', license_plate: '', sts: '', vehicle_type: 'Грузовой', category: '' };
    };

    const saveItem = async (type, payload) => {
      const url = `/api/${type}s`;
      const method = payload.id ? 'PUT' : 'POST';
      try {
        const res = await fetch(url, {
          method,
          headers: { 'Content-Type': 'application/json' },
          credentials: 'include',
          body: JSON.stringify(payload)
        });
        
        if (res.status === 401) {
          isAuthenticated.value = false;
          window.location.href = '/login.html';
          return;
        }
        
        await loadData();
        clearForm(type);
      } catch (e) {
        alert('Ошибка сохранения!');
      }
    };

    const deleteItem = async (type, id) => {
      if (!confirm('Удалить запись?')) return;
      try {
        const res = await fetch(`/api/${type}s`, {
          method: 'DELETE',
          headers: { 'Content-Type': 'application/json' },
          credentials: 'include',
          body: JSON.stringify({ id })
        });
        
        if (res.status === 401) {
          isAuthenticated.value = false;
          window.location.href = '/login.html';
          return;
        }
        
        await loadData();
      } catch (e) {
        alert('Ошибка удаления!');
      }
    };

    const saveSettings = async () => {
      try {
        const res = await fetch('/api/settings', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          credentials: 'include',
          body: JSON.stringify(companySettings.value)
        });
        
        if (res.status === 401) {
          isAuthenticated.value = false;
          window.location.href = '/login.html';
          return;
        }
        
        statusMessage.value = 'Реквизиты сохранены';
        setTimeout(() => statusMessage.value = '', 2000);
      } catch (e) {
        alert('Ошибка сохранения реквизитов');
      }
    };

    const saveDefaults = async () => {
      try {
        const res = await fetch('/api/defaults', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          credentials: 'include',
          body: JSON.stringify(defaultValues.value)
        });
        
        if (res.status === 401) {
          isAuthenticated.value = false;
          window.location.href = '/login.html';
          return;
        }
        
        statusMessage.value = 'Значения по умолчанию сохранены';
        setTimeout(() => statusMessage.value = '', 2000);
      } catch (e) {
        alert('Ошибка сохранения значений по умолчанию');
      }
    };

    return {
      loading,
      isAuthenticated,
      user,
      userInitials,
      currentTab, 
      statusMessage,
      drivers, 
      vehicles, 
      companySettings, 
      defaultValues,
      selectedDriver, 
      selectedVehicle, 
      selectedDate,
      tractorMode, 
      isTractorSelected, 
      selectedVehicleObj,
      taskRows, 
      addTaskRow, 
      removeTaskRow,
      fieldRows, 
      addFieldRow, 
      removeFieldRow,
      printQueue, 
      addToQueue, 
      removeFromQueue, 
      printBatch, 
      saveSettings, 
      saveDefaults,
      formDriver, 
      formVehicle,
      editDriver, 
      editVehicle, 
      clearForm, 
      saveItem, 
      deleteItem
    };
  }
};

createApp(App).mount('#app');
