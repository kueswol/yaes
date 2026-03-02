# AI Prompts:

- Einleitung:
```text
Du bist ein Software Engineer und Experte in Rust. Ich bin IT System Engineer, habe eine Ausbildung vor über 20 Jahren zum Fachinformatiker Anwendungsentwicklung gemacht und dort C/C++ und Visual Basic gelernt. In der Arbeit mache ich sehr viel mit Powershell. Kenne aber inzwischen auch C# etwas besser. Jetzt möchte ich Rust lernen.
Das möchte ich konkret an dem Beispiel eines "Evolution Simulator". In dem Spiel gibt es eine zweidimensionale Welt in der Kreaturen leben.
Kreaturen haben ein Gehirn, welches ein einfaches neuronales Netz mit Binärer Quantisierung ist - das Spiel soll später auf ganz normalen CPUs sehr performant laufen!
Das neuronale Netz möchte ich auch komplett selbst implementieren - im Grunde nach dem NEAT Algorithmus. Wenn Kreaturen sich vermehren, erfolgt eine Mutation - im Grunde wie bei der echten Evolution. Im Rahmen dieser Mutationen soll das neuronale Netzwerk zusätzliche Knoten, Verbindungen oder Gewichte bekommen können.
Um näher an der Realität zu sein, würde ich diese Kreaturen (und speziell das Gehirn) gerne über einen String definieren, der im Grunde der DNA entspricht. Aus den dort kodierten "Genen" soll sich dann eben das Gehirn und deren Verbindungen sowie Gewichte ableiten. Gegebenenfalls werde ich später auch versuchen das ganze mit Multithreading auszubauen.
```

# Projektplan:

## Phasen:

### Phase 1: Grundstrukturen (Jetzt)

- `struct Creature` mit Position, Energie, etc.
- `struct Brain` - das neuronale Netz
- `struct NeuralNode` und `struct Connection` - Knoten und Verbindungen

### Phase 2: DNA/Genotyp

- `struct DNA` - String-basierte Kodierung
- Parser: String → Brain-Struktur
- Mutations-Logik

### Phase 3: Simulation-Welt

- `struct World` - verwaltet alle Kreaturen
- Simulationsloop: Update, Bewegung, Reproduktion

### Phase 4: Optimierungen

- Binäre Quantisierung für Performance
- Multithreading für viele Kreaturen