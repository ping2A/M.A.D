import type { PolicyContentCatalog } from "./types";

export const policyContentFr: PolicyContentCatalog = {
  pillars: {
    cybersecurity_dlp: {
      name: "Cybersécurité et prévention des pertes de données",
      description:
        "Application de la conteneurisation reposant sur le matériel, de l'accès conditionnel Zero Trust via les fournisseurs d'identité, et de la détection et remédiation automatisées des compromissions OS (jailbreak/root).",
    },
    dfir: {
      name: "Forensique numérique et réponse aux incidents",
      description:
        "Isolation réseau à distance préservant la mémoire volatile, triage non destructif sans alerte utilisateur, et streaming API direct des journaux d'audit vers le SIEM d'entreprise.",
    },
    platform_os: {
      name: "Plateforme et prise en charge native des OS",
      description:
        "Prise en charge complète d'Apple Business Manager avec mode supervisé iOS, et d'Android Enterprise (profil professionnel, COBO, kiosque dédié) avec OEMConfig.",
    },
  },
  requirements: {
    "dlp-001": {
      title: "Conteneurisation professionnel/personnel reposant sur le matériel",
      description:
        "La solution MDM doit imposer une séparation professionnel/personnel reposant sur le matériel via des conteneurs natifs (applications gérées iOS / profil professionnel Android).",
      evaluation_method:
        "Déployer des appareils de test avec profil professionnel (Android) et inscription utilisateur / applications gérées (iOS). Tenter une exfiltration inter-conteneurs (presse-papiers, partage de fichiers, capture d'écran). Vérifier que les clés de chiffrement sont liées au matériel (TEE/Secure Enclave).",
      technical_criteria:
        "Android : Android Enterprise Work Profile, Device Policy Controller (DPC), android.app.admin.DevicePolicyManager. iOS : applications gérées, VPN par application, NSFileProtectionComplete, restrictions MDM ManagedOpenIn.",
    },
    "dlp-002": {
      title: "Intégration IdP pour l'accès conditionnel Zero Trust",
      description:
        "Intégration aux fournisseurs d'identité d'entreprise (Entra ID, Okta, etc.) pour des politiques d'accès conditionnel basées sur la conformité des appareils.",
      evaluation_method:
        "Configurer une règle d'accès conditionnel basée sur la conformité IdP. Inscrire des appareils conformes et non conformes. Vérifier l'octroi/refus d'accès aux ressources d'entreprise (M365, applications internes) selon l'état de conformité remonté par la MDM.",
      technical_criteria:
        "API Microsoft Graph deviceManagement, Okta Device Access, revendications SAML/OIDC appareil, charges de politique de conformité (détection jailbreak, version OS, état du chiffrement).",
    },
    "dlp-003": {
      title: "Détection de compromission OS et remédiation automatisée",
      description:
        "Détection immédiate du jailbreak/root avec actions de remédiation automatisées (quarantaine, effacement des données professionnelles, blocage d'accès).",
      evaluation_method:
        "Soumettre un appareil rooté/jailbreaké à la politique de conformité. Mesurer le délai de détection et vérifier le déclenchement automatique (effacement sélectif, effacement complet ou blocage d'accès) sans intervention manuelle.",
      technical_criteria:
        "iOS : détection jailbreak via conformité MDM (supervisé), SafetyNet/DCIM sur Android, signaux API RootBeer, remédiation automatisée via WipeDevice ou action retire via Graph API / REST éditeur.",
    },
    "dfir-001": {
      title: "Isolation réseau à distance avec préservation de la mémoire volatile",
      description:
        "Capacité d'isoler le réseau à distance tout en préservant la mémoire volatile pour l'analyse forensique lors d'une réponse à incident.",
      evaluation_method:
        "Déclencher l'isolation réseau sur un appareil de test actif exécutant des applications au premier plan. Vérifier le blocage TCP/UDP tout en maintenant l'appareil sous tension sans effacement de la RAM (pas de redémarrage, pas d'effacement). Confirmer la réversibilité pour l'imagerie forensique.",
      technical_criteria:
        "VPN par application en mode blocage, règles pare-feu via charge réseau MDM (iOS), VPN always-on Android en mode blocage, ou API « lockdown » propriétaire. Ne doit pas invoquer RebootDevice ou EraseDevice.",
    },
    "dfir-002": {
      title: "Triage silencieux non destructif",
      description:
        "Collecte des journaux système, inventaires d'applications et télémétrie appareil sans alerter ni perturber l'utilisateur final.",
      evaluation_method:
        "Lancer une collecte de journaux à distance et un inventaire d'applications sur un appareil de test. Confirmer l'absence de notification visible, d'interruption UI et de redémarrage. Valider l'exhaustivité des journaux (syslog, crash logs, liste des applications installées).",
      technical_criteria:
        "iOS : flux de journaux MDM (supervisé), récupération de journaux de diagnostic. Android : bugreport via Device Owner, inventaire via ApplicationReport API. Les opérations doivent utiliser le canal MDM en arrière-plan, sans invite utilisateur.",
    },
    "dfir-003": {
      title: "API de streaming des journaux d'audit vers SIEM",
      description:
        "Streaming API direct des journaux d'audit administratifs et de la télémétrie appareil vers les plateformes SIEM d'entreprise (Splunk, Sentinel, etc.).",
      evaluation_method:
        "Activer l'export des journaux d'audit vers un point de terminaison SIEM de test. Effectuer des actions admin (changement de politique, effacement, inscription) et vérifier l'arrivée des événements via API/webhook dans le SLA défini (< 5 min). Confirmer le mapping des champs vers CEF/ECS.",
      technical_criteria:
        "API REST ou flux syslog pour les événements d'audit admin, changements de conformité, inscriptions. Doit prendre en charge Splunk HEC, connecteur Azure Sentinel ou webhook HTTPS générique avec JSON structuré.",
    },
    "plat-001": {
      title: "Apple Business Manager et mode supervisé iOS",
      description:
        "Inscription et gestion complètes via ABM/ADE avec toutes les capacités de politique en mode supervisé.",
      evaluation_method:
        "Inscrire un appareil via l'inscription automatisée ABM (ADE). Vérifier le drapeau de supervision, l'attribution du profil DEP et la capacité à pousser toutes les restrictions réservées au mode supervisé (verrouillage d'app, installation silencieuse, kiosque, flux de journaux).",
      technical_criteria:
        "API Apple Business Manager, inscription MDM DEP, propriété IsSupervised, profils de configuration avec restrictions PayloadType disponibles uniquement en mode supervisé (ex. com.apple.applicationaccess).",
    },
    "plat-002": {
      title: "Modèles de déploiement Android Enterprise",
      description:
        "Prise en charge des modes profil professionnel, COBO (Corporate-Owned Business Only) et kiosque dédié.",
      evaluation_method:
        "Inscrire trois appareils de test : BYOD profil professionnel, COBO Device Owner et kiosque dédié (COSU). Vérifier le ciblage des politiques, le déploiement d'applications et les capacités de gestion selon le mode.",
      technical_criteria:
        "Inscription Android Enterprise zero-touch / QR / NFC, provisionnement Device Owner, configurations gérées, mode kiosque COSU via lock task packages, API android.management (Google AMAPI) ou API EMM éditeur.",
    },
    "plat-003": {
      title: "Prise en charge OEMConfig",
      description:
        "Intégration OEMConfig native pour l'application de politiques spécifiques aux constructeurs Android majeurs (Samsung Knox, Zebra, etc.).",
      evaluation_method:
        "Déployer l'application de configuration gérée OEMConfig sur des appareils Samsung et Zebra de test. Pousser des restrictions OEM (conteneur Knox, Zebra MX) via MDM et vérifier l'application au niveau matériel/firmware.",
      technical_criteria:
        "Configurations gérées Android Enterprise (RESTRICTED_KEYSET), politiques Samsung Knox MDM SDK, schéma XML Zebra OEMConfig, paires clé-valeur de configuration d'application gérée via MDM ApplicationPolicy.",
    },
  },
};
